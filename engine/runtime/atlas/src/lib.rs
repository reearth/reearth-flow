mod damage;
mod error;
mod pack;
mod skyline;

use std::collections::HashMap;
use std::path::PathBuf;

use damage::{collect_damage, Rect, TextureDamage};
pub use error::{AtlasError, Result};
use image::RgbaImage;

pub type PolygonUVs = Vec<[f64; 2]>;
pub type TextureUVs = Vec<PolygonUVs>;

#[derive(Debug, Clone)]
pub struct TextureMaterial {
    pub path: PathBuf,
    pub uvs: TextureUVs,
}

/// Result of a pure layout pass — no image I/O, no blitting.
pub struct LayoutPlan {
    pub atlas_width: u32,
    pub atlas_height: u32,
    /// Downsample factor applied (1 = no downsampling, 2 = half-res, …).
    pub downsample: u32,
    /// Atlas-space top-left `(x, y)` for each input rect, in input order.
    pub placements: Vec<(u32, u32)>,
}

pub struct BuiltAtlas {
    pub image: Option<RgbaImage>,
    pub remapped_uvs: Vec<Option<TextureUVs>>,
}

pub const MAX_DOWNSAMPLE_K: u32 = 13;

struct RemapContext {
    texture_size: (u32, u32),
    damage: Rect,
    frame: Rect,
    atlas_size: (f64, f64),
    downsample: u32,
}

fn remap_uv(u: f64, v: f64, ctx: &RemapContext) -> [f64; 2] {
    let scale = ctx.downsample as f64;
    let px = u * ctx.texture_size.0 as f64 - ctx.damage.x as f64;
    let py = (1.0 - v) * ctx.texture_size.1 as f64 - ctx.damage.y as f64;
    [
        (px / scale + ctx.frame.x as f64) / ctx.atlas_size.0,
        1.0 - (py / scale + ctx.frame.y as f64) / ctx.atlas_size.1,
    ]
}

fn empty_atlas(materials: &[TextureMaterial]) -> BuiltAtlas {
    BuiltAtlas {
        image: None,
        remapped_uvs: materials.iter().map(|_| None).collect(),
    }
}

fn texture_dimensions(damage_list: &[(PathBuf, TextureDamage)]) -> HashMap<String, (u32, u32)> {
    damage_list
        .iter()
        .map(|(path, td)| (path.to_string_lossy().into_owned(), (td.width, td.height)))
        .collect()
}

fn remap_polygon_uvs(
    poly_uvs: &PolygonUVs,
    texture_size: (u32, u32),
    damage: Rect,
    frame: Rect,
    downsample: u32,
    atlas_size: (f64, f64),
) -> PolygonUVs {
    let ctx = RemapContext {
        texture_size,
        damage,
        frame,
        atlas_size,
        downsample,
    };
    poly_uvs
        .iter()
        .map(|&[u, v]| remap_uv(u, v, &ctx))
        .collect()
}

fn build_remapped_uvs(
    materials: &[TextureMaterial],
    damage_list: &[(PathBuf, TextureDamage)],
    texture_frames: &pack::TextureFrames,
    downsample: u32,
    atlas_size: (f64, f64),
) -> Vec<Option<TextureUVs>> {
    let texture_sizes = texture_dimensions(damage_list);
    let damage_by_path: HashMap<_, _> = damage_list
        .iter()
        .map(|(path, td)| (path.to_string_lossy().into_owned(), td))
        .collect();

    materials
        .iter()
        .map(|mat| {
            let path = mat.path.to_string_lossy().into_owned();
            let frames = texture_frames.get(&path)?;
            let damage = damage_by_path.get(&path)?;
            let texture_size = texture_sizes.get(&path).copied().unwrap_or((1, 1));
            Some(
                mat.uvs
                    .iter()
                    .enumerate()
                    .map(|(polygon_idx, poly_uvs)| {
                        let region_idx = damage.polygon_regions[polygon_idx];
                        let (damage, frame) = frames[region_idx];
                        remap_polygon_uvs(
                            poly_uvs,
                            texture_size,
                            damage,
                            frame,
                            downsample,
                            atlas_size,
                        )
                    })
                    .collect(),
            )
        })
        .collect()
}

/// Returns the minimum k such that `side / 2^k <= max`, or `MAX_DOWNSAMPLE_K + 1` if none exists.
fn needed_k(side: u32, max: u32) -> u32 {
    if side <= max {
        return 0;
    }
    let ratio = side.div_ceil(max);
    if ratio > (1u32 << MAX_DOWNSAMPLE_K) {
        return MAX_DOWNSAMPLE_K + 1;
    }
    ratio.next_power_of_two().trailing_zeros()
}

/// Compute the layout for a set of textures given only their dimensions.
/// No image files are read; no pixels are blitted.
/// Useful for efficiency benchmarks and layout-only unit tests.
pub fn plan_layout(dims: &[(u32, u32)], max_atlas_size: u32) -> Result<LayoutPlan> {
    if dims.is_empty() {
        return Ok(LayoutPlan {
            atlas_width: 1,
            atlas_height: 1,
            downsample: 1,
            placements: vec![],
        });
    }
    // `virtual_w/h` is the estimated canvas in original (pre-downsampling) pixel space.
    // Doubling grows the canvas at k=0; once the larger dimension exceeds max_atlas_size,
    // needed_k increments to keep the physical atlas within bounds. Both k and canvas are derived.
    let (mut virtual_w, mut virtual_h) = pack::estimate_atlas_size_from_dims(dims, 0);
    loop {
        let k = needed_k(virtual_w.max(virtual_h), max_atlas_size);
        if k > MAX_DOWNSAMPLE_K {
            break;
        }
        let canvas = (virtual_w.min(max_atlas_size), virtual_h.min(max_atlas_size));
        if let Some((used_w, used_h, placements)) = pack::try_layout_rects(dims, k, canvas) {
            return Ok(LayoutPlan {
                atlas_width: used_w,
                atlas_height: used_h,
                downsample: 1u32 << k,
                placements,
            });
        }
        virtual_w = virtual_w.saturating_mul(2);
        virtual_h = virtual_h.saturating_mul(2);
    }
    Err(AtlasError::builder(format!(
        "Texture atlas does not fit within {}x{} even at downsample factor 2^{}",
        max_atlas_size, max_atlas_size, MAX_DOWNSAMPLE_K
    )))
}

/// Pack `materials` into an atlas image and return remapped UVs.
/// `remapped_uvs[i]` is `Some(remapped_uvs)` if `materials[i]` was packed, `None` if excluded.
pub fn build_atlas(materials: &[TextureMaterial], max_atlas_size: u32) -> Result<BuiltAtlas> {
    // Stage 1: collect damage rects (reads image headers only).
    let damage_list = collect_damage(materials)?;
    if damage_list.is_empty() {
        return Ok(empty_atlas(materials));
    }

    // Stage 2: plan layout (pure — no I/O, no blitting).
    let dims: Vec<(u32, u32)> = damage_list
        .iter()
        .flat_map(|(_, td)| td.rects.iter().map(|r| (r.w, r.h)))
        .collect();
    let plan = plan_layout(&dims, max_atlas_size)?;

    // Stage 3: blit using pre-computed placements — no second layout pass.
    let (atlas, texture_frames) = pack::blit(
        &damage_list,
        plan.downsample,
        (plan.atlas_width, plan.atlas_height),
        &plan.placements,
    )?;
    let atlas_size = (atlas.width() as f64, atlas.height() as f64);
    let remapped_uvs = build_remapped_uvs(
        materials,
        &damage_list,
        &texture_frames,
        plan.downsample,
        atlas_size,
    );
    Ok(BuiltAtlas {
        image: if remapped_uvs.iter().any(Option::is_some) {
            Some(atlas)
        } else {
            None
        },
        remapped_uvs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn make_material(path: PathBuf, uvs: Vec<(f64, f64)>) -> TextureMaterial {
        TextureMaterial {
            path,
            uvs: vec![uvs.into_iter().map(|(u, v)| [u, v]).collect()],
        }
    }

    #[test]
    fn test_build_atlas_uv_mapping() {
        use image::{ImageBuffer, Rgb, RgbImage};

        let temp_dir = TempDir::new().unwrap();

        let img1: RgbImage = ImageBuffer::from_fn(256, 256, |x, y| Rgb([x as u8, y as u8, 10u8]));
        img1.save(temp_dir.path().join("texture1.png")).unwrap();

        let img2: RgbImage = ImageBuffer::from_fn(256, 256, |x, y| Rgb([x as u8, y as u8, 20u8]));
        img2.save(temp_dir.path().join("texture2.png")).unwrap();

        let path1 = temp_dir.path().join("texture1.png");
        let path2 = temp_dir.path().join("texture2.png");

        let materials = vec![
            make_material(path1, vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]),
            make_material(path2, vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]),
        ];

        let built = build_atlas(&materials, 8192).unwrap();
        assert_eq!(built.remapped_uvs.len(), 2);
        assert!(built.remapped_uvs.iter().all(|entry| entry.is_some()));
        assert!(built.image.as_ref().unwrap().width() > 0);
        assert!(built.image.as_ref().unwrap().height() > 0);
    }

    // A 16x16 texture at max downsample (k=4, factor=16) shrinks to 1x1 px, which the packer
    // places in a 3x3 slot (1px content + 1px extrusion on each side).  Four such slots tile
    // into a 6x6 atlas (2×2 arrangement), so four textures fit exactly; a fifth cannot.

    // Bleeding test: solid-color textures are used so any inter-region bleed produces a
    // detectably wrong pixel.  64x64 sources with max_atlas_size=32 forces k=3 (downsample×8).
    // Downsampling a solid color is lossless (all pixels identical), so exact color equality holds.
    #[test]
    fn test_no_bleeding_between_packed_regions() {
        use image::{Rgba, RgbaImage};

        let temp_dir = TempDir::new().unwrap();
        let colors = [
            Rgba([255u8, 0, 0, 255]),
            Rgba([0, 255, 0, 255]),
            Rgba([0, 0, 255, 255]),
            Rgba([255, 255, 0, 255]),
        ];

        let materials: Vec<_> = colors
            .iter()
            .enumerate()
            .map(|(i, &color)| {
                let mut img = RgbaImage::new(64, 64);
                for pixel in img.pixels_mut() {
                    *pixel = color;
                }
                let path = temp_dir.path().join(format!("t{i}.png"));
                img.save(&path).unwrap();
                make_material(path, vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)])
            })
            .collect();

        let built = build_atlas(&materials, 32).unwrap();
        let atlas = built.image.as_ref().unwrap();
        let aw = atlas.width() as f64;
        let ah = atlas.height() as f64;

        for (mat_idx, (color, remapped)) in colors.iter().zip(built.remapped_uvs.iter()).enumerate()
        {
            let poly_uvs = remapped
                .as_ref()
                .unwrap_or_else(|| panic!("material {mat_idx} not packed"));

            // Vertices are (0,0),(1,0),(1,1),(0,1); v3=(u=0,v=1) is atlas top-left,
            // v1=(u=1,v=0) is atlas bottom-right.
            let [tl_u, tl_v] = poly_uvs[0][3];
            let [br_u, br_v] = poly_uvs[0][1];
            let x0 = (tl_u * aw).round() as u32;
            let y0 = ((1.0 - tl_v) * ah).round() as u32;
            let x1 = (br_u * aw).round() as u32;
            let y1 = ((1.0 - br_v) * ah).round() as u32;
            assert!(x1 > x0 && y1 > y0, "empty frame for material {mat_idx}");

            // Content area and 1px extrusion ring must all be the expected solid color.
            let ex0 = x0.saturating_sub(1);
            let ey0 = y0.saturating_sub(1);
            let ex1 = (x1 + 1).min(atlas.width());
            let ey1 = (y1 + 1).min(atlas.height());
            for y in ey0..ey1 {
                for x in ex0..ex1 {
                    assert_eq!(
                        *atlas.get_pixel(x, y),
                        *color,
                        "bleed at ({x},{y}) for material {mat_idx}"
                    );
                }
            }
        }
    }

    #[test]
    fn test_four_16x16_fit_in_6x6_atlas() {
        let plan = plan_layout(&vec![(16, 16); 4], 6).unwrap();
        assert!(plan.atlas_width <= 6 && plan.atlas_height <= 6);
    }

    #[test]
    fn test_five_16x16_do_not_fit_in_6x6_atlas() {
        assert!(plan_layout(&vec![(16, 16); 5], 6).is_err());
    }

    #[test]
    fn test_plan_layout_triggers_downsampling() {
        let plan = plan_layout(&vec![(1536, 1536); 2], 2048).unwrap();
        assert!(plan.downsample > 1);
        assert!(plan.atlas_width <= 2048 && plan.atlas_height <= 2048);
    }
}
