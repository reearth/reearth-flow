//! Multi-page atlas packing: each texture at its own scale, overflow spilling onto
//! further pages rather than downsampling the whole set as [`crate::build_atlas`] does.
//! Assumes the top-left UV origin of the new-geometry writer.

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, Rgba, RgbaImage};

use crate::blit::fill_frame_extrusion;
use crate::damage::collect_damage;
use crate::skyline::SkylinePacker;
use crate::{remap_polygon_uvs, AtlasError, PolygonUVs, Rect, Result, TextureInput};

/// Decoded-source-image cache; share one across calls so each file is decoded once.
#[derive(Default)]
pub struct TextureCache {
    images: HashMap<PathBuf, DynamicImage>,
}

impl TextureCache {
    /// Decode `path` once, then serve it from memory on later calls.
    fn get(&mut self, path: &Path) -> Result<&DynamicImage> {
        match self.images.entry(path.to_path_buf()) {
            Entry::Occupied(e) => Ok(e.into_mut()),
            Entry::Vacant(e) => {
                let image = image::open(path).map_err(|err| {
                    AtlasError::builder(format!(
                        "Failed to open texture '{}': {err}",
                        path.display()
                    ))
                })?;
                Ok(e.insert(image))
            }
        }
    }
}

/// Where one source polygon's UVs landed in the built atlas.
pub struct PolygonPlacement {
    /// Index into [`MultiPageAtlas::pages`].
    pub page: usize,
    /// Atlas-space UVs, parallel to the source polygon's UVs.
    pub uvs: PolygonUVs,
}

/// How a page must be addressed outside `[0, 1]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageWrap {
    /// Packed page: several textures side by side.
    Clamp,
    /// One tiling texture, bound whole.
    Repeat,
}

/// Page images plus, per input material, each polygon's placement.
pub struct MultiPageAtlas {
    pub pages: Vec<RgbaImage>,
    /// Parallel to `pages`.
    pub wrap: Vec<PageWrap>,
    /// Parallel to the input `materials`, then to each one's `TextureInput::uvs`.
    pub remapped: Vec<Vec<PolygonPlacement>>,
}

/// A damage region to place: its source rect plus the size it takes in the atlas.
struct RegionJob {
    /// Index into `damage_list`.
    damage: usize,
    src: Rect,
    target_w: u32,
    target_h: u32,
}

/// Upper bound (pixels) for `max_atlas_size` and `extrusion`; a page that large is
/// already ~17 GB of RGBA, so anything beyond it is a misconfiguration.
pub const MAX_ATLAS_DIMENSION: u32 = 65_536;

/// Pack `materials` into atlas pages, giving any tiling texture a page of its own.
/// `Ok(None)` when there is nothing to pack; `Err` when `max_atlas_size` is 0 or it
/// or `extrusion` exceeds [`MAX_ATLAS_DIMENSION`].
pub fn build_atlas_multipage(
    materials: &[TextureInput],
    max_atlas_size: u32,
    extrusion: u32,
    block_align: u32,
    wrap_tolerance: f64,
    cache: &mut TextureCache,
) -> Result<Option<MultiPageAtlas>> {
    if max_atlas_size == 0 {
        return Err(AtlasError::builder("atlas size must be at least 1"));
    }
    // Capping both keeps the packing arithmetic clear of `u32` overflow.
    if max_atlas_size > MAX_ATLAS_DIMENSION || extrusion > MAX_ATLAS_DIMENSION {
        return Err(AtlasError::builder(format!(
            "atlas size ({max_atlas_size}) and extrusion ({extrusion}) must each be \
             at most {MAX_ATLAS_DIMENSION}"
        )));
    }
    let block_align = block_align.max(1);
    // Snap the gap too, so every reserved footprint stays on the block grid.
    let extrusion = extrusion.div_ceil(block_align) * block_align;

    // Past `wrap_tolerance` a UV is tiling, not drift; the sampler wraps such a
    // texture, so it cannot share a page.
    let tiling: Vec<bool> = materials
        .iter()
        .map(|mat| tiles(mat, wrap_tolerance))
        .collect();
    let damage_list = collect_damage(
        materials
            .iter()
            .zip(&tiling)
            .filter(|(_, &t)| !t)
            .map(|(mat, _)| mat),
    )?;

    // One scale per source path: the largest, i.e. the least downsampling asked for.
    let mut scale_by_path: HashMap<&PathBuf, f64> = HashMap::new();
    for mat in materials {
        let scale = mat.scale.clamp(f64::MIN_POSITIVE, 1.0);
        scale_by_path
            .entry(&mat.path)
            .and_modify(|e| *e = e.max(scale))
            .or_insert(scale);
    }

    // Flatten every damage region into a placement job, recording where each lands.
    let mut jobs: Vec<RegionJob> = Vec::new();
    let mut region_job: Vec<Vec<usize>> = Vec::with_capacity(damage_list.len());
    for (di, (path, td)) in damage_list.iter().enumerate() {
        let scale = scale_by_path.get(path).copied().unwrap_or(1.0);
        let mut per_region = Vec::with_capacity(td.rects.len());
        for &src in &td.rects {
            let mut w = ((src.w as f64) * scale).round().max(1.0) as u32;
            let mut h = ((src.h as f64) * scale).round().max(1.0) as u32;
            if w > max_atlas_size || h > max_atlas_size {
                // Bigger than a whole page before packing even starts.
                let shrink = max_atlas_size as f64 / w.max(h) as f64;
                let sw = ((w as f64) * shrink)
                    .round()
                    .clamp(1.0, max_atlas_size as f64) as u32;
                let sh = ((h as f64) * shrink)
                    .round()
                    .clamp(1.0, max_atlas_size as f64) as u32;
                tracing::warn!(
                    "reearth-flow-atlas: region {w}x{h} of '{}' exceeds atlas size \
                     {max_atlas_size}; force-shrinking to {sw}x{sh}",
                    path.display()
                );
                w = sw;
                h = sh;
            }
            // Round up so no compression block straddles a region boundary.
            let align_up = |v: u32| (v.div_ceil(block_align) * block_align).min(max_atlas_size);
            w = align_up(w);
            h = align_up(h);
            per_region.push(jobs.len());
            jobs.push(RegionJob {
                damage: di,
                src,
                target_w: w,
                target_h: h,
            });
        }
        region_job.push(per_region);
    }

    // Next-fit, tallest-first; earlier pages are never revisited.
    let mut order: Vec<usize> = (0..jobs.len()).collect();
    order.sort_by(|&a, &b| {
        jobs[b]
            .target_h
            .cmp(&jobs[a].target_h)
            .then(jobs[b].target_w.cmp(&jobs[a].target_w))
    });
    let mut packers: Vec<SkylinePacker> = Vec::new();
    let mut placement: Vec<Option<(usize, Rect)>> = vec![None; jobs.len()];
    for &j in &order {
        let (w, h) = (jobs[j].target_w, jobs[j].target_h);
        let frame = match packers.last_mut().and_then(|p| p.pack(w, h)) {
            Some(frame) => frame,
            None => {
                let mut packer = SkylinePacker::new(max_atlas_size, max_atlas_size, extrusion);
                let frame = packer
                    .pack(w, h)
                    .expect("a region clamped to `max_atlas_size` always fits an empty page");
                packers.push(packer);
                frame
            }
        };
        placement[j] = Some((packers.len() - 1, frame));
    }

    // Blit: crop each source region, resize to its placement, copy in, extrude.
    let mut pages: Vec<RgbaImage> = packers
        .iter()
        .map(|p| RgbaImage::from_pixel(p.width(), p.height(), Rgba([0, 0, 0, 0])))
        .collect();
    for (j, job) in jobs.iter().enumerate() {
        let (page, frame) = placement[j].expect("every job was placed");
        let path = &damage_list[job.damage].0;
        let source = cache.get(path)?;
        let mut crop = source
            .crop_imm(job.src.x, job.src.y, job.src.w, job.src.h)
            .to_rgba8();
        if (frame.w, frame.h) != (job.src.w, job.src.h) {
            crop = image::imageops::resize(&crop, frame.w, frame.h, FilterType::Triangle);
        }
        pages[page]
            .copy_from(&crop, frame.x, frame.y)
            .map_err(|_| AtlasError::builder("Internal bug: failed to copy texture into atlas"))?;
        fill_frame_extrusion(&mut pages[page], frame, extrusion);
    }
    let mut wrap = vec![PageWrap::Clamp; pages.len()];

    // One page per tiling texture, holding the whole image so its UVs address it unchanged.
    let mut tiling_page: HashMap<&PathBuf, usize> = HashMap::new();
    for (mat, _) in materials.iter().zip(&tiling).filter(|(_, &t)| t) {
        if tiling_page.contains_key(&mat.path) {
            continue;
        }
        let scale = scale_by_path.get(&mat.path).copied().unwrap_or(1.0);
        let page = whole_page(cache.get(&mat.path)?, scale, max_atlas_size);
        tiling_page.insert(&mat.path, pages.len());
        pages.push(page);
        wrap.push(PageWrap::Repeat);
    }

    if pages.is_empty() {
        return Ok(None);
    }

    // Remap each material's UVs into atlas space, tagged with its page.
    let mut di_by_path: HashMap<&PathBuf, usize> = HashMap::new();
    for (di, (path, _)) in damage_list.iter().enumerate() {
        di_by_path.insert(path, di);
    }
    let remapped = materials
        .iter()
        .zip(&tiling)
        .map(|(mat, &t)| {
            // The page is the texture, so its UVs already address it.
            if t {
                let page = tiling_page[&mat.path];
                return mat
                    .uvs
                    .iter()
                    .map(|uvs| PolygonPlacement {
                        page,
                        uvs: uvs.clone(),
                    })
                    .collect();
            }
            let Some(&di) = di_by_path.get(&mat.path) else {
                return Vec::new(); // material contributed no polygons
            };
            let (_, td) = &damage_list[di];
            mat.uvs
                .iter()
                .enumerate()
                .map(|(pi, poly_uvs)| {
                    let ri = td.polygon_regions[pi];
                    let job_idx = region_job[di][ri];
                    let (page, frame) = placement[job_idx].expect("placed");
                    let src = jobs[job_idx].src;
                    let page_size = (pages[page].width() as f64, pages[page].height() as f64);
                    let uvs = remap_polygon_uvs(
                        poly_uvs,
                        (td.src_width, td.src_height),
                        src,
                        frame,
                        page_size,
                    );
                    PolygonPlacement { page, uvs }
                })
                .collect()
        })
        .collect();

    Ok(Some(MultiPageAtlas {
        pages,
        wrap,
        remapped,
    }))
}

fn tiles(mat: &TextureInput, wrap_tolerance: f64) -> bool {
    let unit = -wrap_tolerance..=1.0 + wrap_tolerance;
    mat.uvs
        .iter()
        .flatten()
        .any(|[u, v]| !unit.contains(u) || !unit.contains(v))
}

/// The whole texture as its own page: `scale` first (never an upscale), then a hard clamp to one page.
fn whole_page(source: &DynamicImage, scale: f64, max_atlas_size: u32) -> RgbaImage {
    let (w, h) = (source.width(), source.height());
    let scale = (scale.clamp(f64::MIN_POSITIVE, 1.0))
        .min(max_atlas_size as f64 / w.max(h) as f64)
        .min(1.0);
    let target = |v: u32| ((v as f64 * scale).round() as u32).clamp(1, max_atlas_size);
    let (tw, th) = (target(w), target(h));
    if (tw, th) == (w, h) {
        source.to_rgba8()
    } else {
        image::imageops::resize(&source.to_rgba8(), tw, th, FilterType::Triangle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn material(path: PathBuf, uvs: Vec<(f64, f64)>, scale: f64) -> TextureInput {
        TextureInput {
            path,
            uvs: vec![uvs.into_iter().map(|(u, v)| [u, v]).collect()],
            scale,
        }
    }

    fn write_texture(dir: &std::path::Path, name: &str, w: u32, h: u32) -> PathBuf {
        let img = RgbaImage::from_pixel(w, h, Rgba([200, 100, 50, 255]));
        let path = dir.join(name);
        img.save(&path).unwrap();
        path
    }

    #[test]
    fn full_scale_single_page() {
        let tmp = TempDir::new().unwrap();
        let a = material(
            write_texture(tmp.path(), "a.png", 64, 64),
            vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            1.0,
        );
        let built = build_atlas_multipage(&[a], 4096, 1, 1, 0.0, &mut TextureCache::default())
            .unwrap()
            .expect("atlas built");
        assert_eq!(built.pages.len(), 1);
        assert_eq!(built.remapped.len(), 1);
        assert_eq!(built.remapped[0].len(), 1);
        assert_eq!(built.remapped[0][0].page, 0);
    }

    #[test]
    fn target_scale_shrinks_placement() {
        let tmp = TempDir::new().unwrap();
        let full = material(
            write_texture(tmp.path(), "full.png", 256, 256),
            vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            1.0,
        );
        let half = TextureInput {
            path: full.path.clone(),
            uvs: full.uvs.clone(),
            scale: 0.5,
        };
        let full_atlas = build_atlas_multipage(
            std::slice::from_ref(&full),
            4096,
            1,
            1,
            0.0,
            &mut TextureCache::default(),
        )
        .unwrap()
        .unwrap();
        let half_atlas = build_atlas_multipage(&[half], 4096, 1, 1, 0.0, &mut TextureCache::default())
            .unwrap()
            .unwrap();
        // Downscaling to 0.5 must yield a smaller page than full resolution.
        assert!(half_atlas.pages[0].width() < full_atlas.pages[0].width());
    }

    #[test]
    fn overflow_spills_to_second_page() {
        let tmp = TempDir::new().unwrap();
        // Two 200x200 regions cannot share a 256-wide/high page, forcing a
        // second page (no downsampling requested).
        let mats: Vec<_> = (0..2)
            .map(|i| {
                material(
                    write_texture(tmp.path(), &format!("t{i}.png"), 200, 200),
                    vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
                    1.0,
                )
            })
            .collect();
        let built = build_atlas_multipage(&mats, 256, 1, 1, 0.0, &mut TextureCache::default())
            .unwrap()
            .expect("atlas built");
        assert_eq!(built.pages.len(), 2);
        let pages: Vec<usize> = built.remapped.iter().map(|m| m[0].page).collect();
        assert_ne!(pages[0], pages[1], "the two regions land on distinct pages");
    }

    #[test]
    fn oversized_texture_force_shrinks_onto_one_page() {
        let tmp = TempDir::new().unwrap();
        // Single 512x512 region, atlas cap 128: must be force-shrunk to fit.
        let mat = material(
            write_texture(tmp.path(), "big.png", 512, 512),
            vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            1.0,
        );
        let built = build_atlas_multipage(&[mat], 128, 1, 1, 0.0, &mut TextureCache::default())
            .unwrap()
            .expect("atlas built");
        assert_eq!(built.pages.len(), 1);
        assert!(built.pages[0].width() <= 128 && built.pages[0].height() <= 128);
    }

    #[test]
    fn tiling_texture_takes_its_own_page_whole() {
        let tmp = TempDir::new().unwrap();
        let packed = material(
            write_texture(tmp.path(), "packed.png", 64, 64),
            vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            1.0,
        );
        let tiled = material(
            write_texture(tmp.path(), "tiled.png", 64, 32),
            vec![(0.0, 0.0), (8.0, 0.0), (8.0, 6.0), (0.0, 6.0)],
            1.0,
        );
        let built =
            build_atlas_multipage(&[packed, tiled], 4096, 1, 1, 0.0, &mut TextureCache::default())
                .unwrap()
                .expect("atlas built");

        let packed_page = built.remapped[0][0].page;
        let tiled_page = built.remapped[1][0].page;
        assert_ne!(packed_page, tiled_page, "a tiling texture shares no page");
        assert_eq!(built.wrap[packed_page], PageWrap::Clamp);
        assert_eq!(built.wrap[tiled_page], PageWrap::Repeat);
        // The page is the texture, so the source UVs address it unchanged.
        assert_eq!(built.pages[tiled_page].dimensions(), (64, 32));
        assert_eq!(
            built.remapped[1][0].uvs,
            vec![[0.0, 0.0], [8.0, 0.0], [8.0, 6.0], [0.0, 6.0]]
        );
    }

    #[test]
    fn tiling_page_obeys_scale_then_atlas_cap() {
        let tmp = TempDir::new().unwrap();
        let path = write_texture(tmp.path(), "big.png", 512, 512);
        let uvs = vec![(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)];

        let scaled = build_atlas_multipage(
            &[material(path.clone(), uvs.clone(), 0.5)],
            4096,
            1,
            1,
            0.0,
            &mut TextureCache::default(),
        )
        .unwrap()
        .unwrap();
        assert_eq!(scaled.pages[0].dimensions(), (256, 256));

        let capped = build_atlas_multipage(
            &[material(path, uvs, 1.0)],
            128,
            1,
            1,
            0.0,
            &mut TextureCache::default(),
        )
        .unwrap()
        .unwrap();
        assert_eq!(capped.pages[0].dimensions(), (128, 128));
    }

    #[test]
    fn nonuniform_rounding_maps_uv_span_to_frame_dims() {
        // A 3x5 region at scale 0.5 rounds to a 2x3 frame: the width ratio
        // (2/3) and height ratio (3/5) differ, so no single scale fits both
        // axes. The old remap divided both axes by src.w/frame.w = 1.5, which
        // is exact horizontally (3/1.5 = 2 px) but stretches the height to
        // 5/1.5 = 3.33 px instead of the frame's 3 px. Per-axis mapping
        // reproduces the frame dimensions exactly. Mapping the UV span back
        // into page pixels must therefore recover 2x3.
        let tmp = TempDir::new().unwrap();
        let mat = material(
            write_texture(tmp.path(), "skew.png", 3, 5),
            vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            0.5,
        );
        let built = build_atlas_multipage(&[mat], 64, 0, 1, 0.0, &mut TextureCache::default())
            .unwrap()
            .expect("atlas built");
        let page_w = built.pages[0].width() as f64;
        let page_h = built.pages[0].height() as f64;

        let uvs = &built.remapped[0][0].uvs;
        let (mut min_u, mut max_u, mut min_v, mut max_v) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
        for &[u, v] in uvs {
            min_u = min_u.min(u);
            max_u = max_u.max(u);
            min_v = min_v.min(v);
            max_v = max_v.max(v);
        }
        let mapped_w = (max_u - min_u) * page_w;
        let mapped_h = (max_v - min_v) * page_h;

        assert!(
            (mapped_w - 2.0).abs() < 1e-6,
            "mapped width {mapped_w}, expected 2.0 (frame.w)"
        );
        assert!(
            (mapped_h - 3.0).abs() < 1e-6,
            "mapped height {mapped_h}, expected 3.0 (frame.h); old single-scale remap yields ~3.33"
        );
    }
}
