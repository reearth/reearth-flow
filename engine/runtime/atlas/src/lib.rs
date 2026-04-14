mod damage;
mod error;
mod pack;
mod skyline;

use std::collections::HashMap;
use std::path::PathBuf;

use damage::{collect_damage, DamageRect, TextureDamage};
pub use error::{AtlasError, Result};
use image::RgbaImage;
use pack::pack_textures;

pub type PolygonUVs = Vec<[f64; 2]>;
pub type TextureUVs = Vec<PolygonUVs>;

#[derive(Debug, Clone)]
pub struct TextureMaterial {
    pub path: PathBuf,
    pub uvs: TextureUVs,
}

pub struct BuiltAtlas {
    pub image: Option<RgbaImage>,
    pub remapped_uvs: Vec<Option<TextureUVs>>,
}

pub const MAX_DOWNSAMPLE_K: u32 = 13;

struct PackedAtlas {
    damage_list: Vec<(PathBuf, damage::TextureDamage)>,
    info: pack::AtlasInfo,
}

struct RemapContext {
    texture_size: (u32, u32),
    damage: DamageRect,
    frame: DamageRect,
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

fn pack_atlas(materials: &[TextureMaterial], max_atlas_size: u32) -> Result<Option<PackedAtlas>> {
    let mut k = 0;
    let mut damage_list = collect_damage(materials, k)?;
    if damage_list.is_empty() {
        return Ok(None);
    }

    // guided_w/h stay on the power-of-2 ladder and drive the retry stepping.
    // The actual canvas passed to the packer is guided clamped to max_atlas_size.
    let (mut guided_w, mut guided_h) = pack::estimate_atlas_size(&damage_list, k);
    loop {
        let current_size = (guided_w.min(max_atlas_size), guided_h.min(max_atlas_size));
        match pack_textures(&damage_list, k, current_size)? {
            pack::PackResult::Packed(info) => return Ok(Some(PackedAtlas { damage_list, info })),
            pack::PackResult::NeedsDownscale => {
                if guided_w < max_atlas_size || guided_h < max_atlas_size {
                    // Actual canvas can still grow — advance the power-of-2 ladder.
                    guided_w = guided_w.saturating_mul(2);
                    guided_h = guided_h.saturating_mul(2);
                } else {
                    // Canvas is fully maxed — must downsample.
                    if k >= MAX_DOWNSAMPLE_K {
                        return Err(AtlasError::builder(format!(
                            "Texture atlas does not fit within {}x{} even at downsample factor 2^{}",
                            max_atlas_size, max_atlas_size, k
                        )));
                    }
                    k += 1;
                    damage_list = collect_damage(materials, k)?;
                    if damage_list.is_empty() {
                        return Ok(None);
                    }
                }
            }
        }
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
    damage: DamageRect,
    frame: DamageRect,
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
    info: &pack::AtlasInfo,
) -> Vec<Option<TextureUVs>> {
    let texture_sizes = texture_dimensions(damage_list);
    let damage_by_path: HashMap<_, _> = damage_list
        .iter()
        .map(|(path, td)| (path.to_string_lossy().into_owned(), td))
        .collect();
    let atlas_size = (info.width as f64, info.height as f64);

    materials
        .iter()
        .map(|mat| {
            let path = mat.path.to_string_lossy().into_owned();
            let frames = info.texture_frames.get(&path)?;
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
                            info.downsample,
                            atlas_size,
                        )
                    })
                    .collect(),
            )
        })
        .collect()
}

/// Pack `materials` into an atlas image and return remapped UVs.
/// `remapped_uvs[i]` is `Some(remapped_uvs)` if `materials[i]` was packed, `None` if excluded.
pub fn build_atlas(materials: &[TextureMaterial], max_atlas_size: u32) -> Result<BuiltAtlas> {
    let Some(packed) = pack_atlas(materials, max_atlas_size)? else {
        return Ok(empty_atlas(materials));
    };
    eprintln!("{:?}", packed.damage_list);

    let remapped_uvs = build_remapped_uvs(materials, &packed.damage_list, &packed.info);
    let PackedAtlas { info, .. } = packed;

    Ok(BuiltAtlas {
        image: if remapped_uvs.iter().any(Option::is_some) {
            Some(info.atlas)
        } else {
            None
        },
        remapped_uvs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    fn create_test_texture(dir: &Path, name: &str, width: u32, height: u32) -> PathBuf {
        use image::{ImageBuffer, Rgb};
        let img = ImageBuffer::<Rgb<u8>, _>::new(width, height);
        let path = dir.join(name);
        img.save(&path).unwrap();
        path
    }

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
        let temp_dir = TempDir::new().unwrap();
        let full_uvs = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        let materials: Vec<_> = (0..4)
            .map(|i| {
                let path = create_test_texture(temp_dir.path(), &format!("t{i}.png"), 16, 16);
                make_material(path, full_uvs.clone())
            })
            .collect();

        let built = build_atlas(&materials, 6).unwrap();
        assert!(built.remapped_uvs.iter().all(|e| e.is_some()));
        let img = built.image.as_ref().unwrap();
        assert!(img.width() <= 6 && img.height() <= 6);
    }

    #[test]
    fn test_five_16x16_do_not_fit_in_6x6_atlas() {
        let temp_dir = TempDir::new().unwrap();
        let full_uvs = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        let materials: Vec<_> = (0..5)
            .map(|i| {
                let path = create_test_texture(temp_dir.path(), &format!("t{i}.png"), 16, 16);
                make_material(path, full_uvs.clone())
            })
            .collect();

        assert!(build_atlas(&materials, 6).is_err());
    }

    #[test]
    fn test_build_atlas_retries_with_downscaling() {
        let temp_dir = TempDir::new().unwrap();
        let max_atlas_size = 2048;
        let path1 = create_test_texture(temp_dir.path(), "large1.png", 1536, 1536);
        let path2 = create_test_texture(temp_dir.path(), "large2.png", 1536, 1536);

        let materials = vec![
            make_material(path1, vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]),
            make_material(path2, vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]),
        ];

        let built = build_atlas(&materials, max_atlas_size).unwrap();

        assert!(built.image.as_ref().unwrap().width() <= max_atlas_size);
        assert!(built.image.as_ref().unwrap().height() <= max_atlas_size);
    }
}
