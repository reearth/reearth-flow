//! Multi-page atlas packing.
//!
//! Unlike [`crate::build_atlas`] — which shrinks every texture by one global
//! factor until the whole set fits a single page — this packs each texture at a
//! caller-chosen target scale (metres-per-pixel driven, computed from geometry)
//! and spills the overflow onto additional pages instead of downsampling
//! further. A single region larger than one page is force-shrunk to fit, with a
//! warning; page count is otherwise unbounded.
//!
//! Assumes the top-left UV origin used by the new-geometry writer (see
//! [`crate::remap_polygon_uvs`], whose v-axis handling is `#[cfg]`-selected).

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, Rgba, RgbaImage};

use crate::blit::fill_frame_extrusion;
use crate::damage::collect_damage;
use crate::skyline::SkylinePacker;
use crate::{remap_polygon_uvs, AtlasError, PolygonUVs, Rect, Result, TextureInput};

/// Extrusion ring (pixels) blitted around each region to stop bilinear bleed;
/// matches the legacy single-page packer.
const EXTRUSION: u32 = 1;

/// Decoded-source-image cache, keyed by path. Source textures are shared across
/// tiles, so decoding is the dominant cost of a tileset build; pass one cache
/// to every [`build_atlas_multipage`] call so each file is decoded once.
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
                    AtlasError::builder(format!("Failed to open texture '{}': {err}", path.display()))
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

/// A packed multi-page atlas: one or more page images plus, per input material,
/// the per-polygon placement (page + remapped UVs).
pub struct MultiPageAtlas {
    pub pages: Vec<RgbaImage>,
    /// Parallel to the input `materials`; each inner vec is parallel to that
    /// material's `TextureInput::uvs`.
    pub remapped: Vec<Vec<PolygonPlacement>>,
}

/// One damage region to place: its source-pixel rect plus the target size it
/// should occupy in the atlas (already scaled and clamped to a single page).
struct RegionJob {
    /// Index into `damage_list`.
    damage: usize,
    src: Rect,
    target_w: u32,
    target_h: u32,
}

/// Pack `materials` into one or more atlas pages. `target_scales` is parallel to
/// `materials`: each is the fraction of native resolution to keep for that
/// texture (`(0, 1]`; `1.0` = full resolution, never upsampled). Materials
/// sharing a path take the largest of their scales.
///
/// Returns `Ok(None)` when there is nothing to pack (no UV polygons).
pub fn build_atlas_multipage(
    materials: &[TextureInput],
    target_scales: &[f64],
    max_atlas_size: u32,
    cache: &mut TextureCache,
) -> Result<Option<MultiPageAtlas>> {
    debug_assert_eq!(materials.len(), target_scales.len());

    let damage_list = collect_damage(materials)?;
    if damage_list.is_empty() {
        return Ok(None);
    }

    // One scale per source path (a path may be referenced by several
    // materials); keep the largest, i.e. the least downsampling any use asks for.
    let mut scale_by_path: HashMap<&PathBuf, f64> = HashMap::new();
    for (mat, &scale) in materials.iter().zip(target_scales) {
        let scale = scale.clamp(f64::MIN_POSITIVE, 1.0);
        scale_by_path
            .entry(&mat.path)
            .and_modify(|e| *e = e.max(scale))
            .or_insert(scale);
    }

    // Flatten every damage region into a placement job, recording where each
    // (damage, region) lands so UV remapping can find it afterwards.
    let usable = max_atlas_size.saturating_sub(2 * EXTRUSION).max(1);
    let mut jobs: Vec<RegionJob> = Vec::new();
    let mut region_job: Vec<Vec<usize>> = Vec::with_capacity(damage_list.len());
    for (di, (path, td)) in damage_list.iter().enumerate() {
        let scale = scale_by_path.get(path).copied().unwrap_or(1.0);
        let mut per_region = Vec::with_capacity(td.rects.len());
        for &src in &td.rects {
            let mut w = ((src.w as f64) * scale).round().max(1.0) as u32;
            let mut h = ((src.h as f64) * scale).round().max(1.0) as u32;
            if w > usable || h > usable {
                // Bigger than a whole page even before packing: shrink to fit,
                // preserving aspect. Geometric error is intentionally left
                // untouched — this is a packing constraint, not user intent.
                let shrink = usable as f64 / w.max(h) as f64;
                let sw = ((w as f64) * shrink).round().clamp(1.0, usable as f64) as u32;
                let sh = ((h as f64) * shrink).round().clamp(1.0, usable as f64) as u32;
                tracing::warn!(
                    "reearth-flow-atlas: region {w}x{h} of '{}' exceeds atlas size \
                     {max_atlas_size}; force-shrinking to {sw}x{sh}",
                    path.display()
                );
                w = sw;
                h = sh;
            }
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

    // First-fit across pages, tallest-first (the skyline packer's preferred
    // order). `pack` leaves a packer untouched when it returns `None`, so
    // probing pages in turn is safe.
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
        let mut placed = None;
        for (page, packer) in packers.iter_mut().enumerate() {
            if let Some(frame) = packer.pack(w, h) {
                placed = Some((page, frame));
                break;
            }
        }
        placement[j] = Some(placed.unwrap_or_else(|| {
            let mut packer = SkylinePacker::new(max_atlas_size, max_atlas_size, EXTRUSION);
            let frame = packer
                .pack(w, h)
                .expect("a region clamped to `usable` always fits an empty page");
            let page = packers.len();
            packers.push(packer);
            (page, frame)
        }));
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
        fill_frame_extrusion(&mut pages[page], frame, EXTRUSION);
    }

    // Remap each material's UVs into atlas space, tagged with its page.
    let mut di_by_path: HashMap<&PathBuf, usize> = HashMap::new();
    for (di, (path, _)) in damage_list.iter().enumerate() {
        di_by_path.insert(path, di);
    }
    let remapped = materials
        .iter()
        .map(|mat| {
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
                    let scale = src.w as f64 / frame.w as f64;
                    let uvs = remap_polygon_uvs(
                        poly_uvs,
                        (td.src_width, td.src_height),
                        src,
                        frame,
                        scale,
                        page_size,
                    );
                    PolygonPlacement { page, uvs }
                })
                .collect()
        })
        .collect();

    Ok(Some(MultiPageAtlas { pages, remapped }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn material(path: PathBuf, uvs: Vec<(f64, f64)>) -> TextureInput {
        TextureInput {
            path,
            uvs: vec![uvs.into_iter().map(|(u, v)| [u, v]).collect()],
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
        );
        let built = build_atlas_multipage(&[a], &[1.0], 4096, &mut TextureCache::default())
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
        );
        let half_path = full.path.clone();
        let half = TextureInput {
            path: half_path,
            uvs: full.uvs.clone(),
        };
        let full_atlas = build_atlas_multipage(std::slice::from_ref(&full), &[1.0], 4096, &mut TextureCache::default())
            .unwrap()
            .unwrap();
        let half_atlas = build_atlas_multipage(&[half], &[0.5], 4096, &mut TextureCache::default())
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
                )
            })
            .collect();
        let built = build_atlas_multipage(&mats, &[1.0, 1.0], 256, &mut TextureCache::default())
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
        );
        let built = build_atlas_multipage(&[mat], &[1.0], 128, &mut TextureCache::default())
            .unwrap()
            .expect("atlas built");
        assert_eq!(built.pages.len(), 1);
        assert!(built.pages[0].width() <= 128 && built.pages[0].height() <= 128);
    }
}
