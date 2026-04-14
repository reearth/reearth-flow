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

    let mut current_size = pack::estimate_atlas_size(&damage_list, k, max_atlas_size);
    loop {
        match pack_textures(&damage_list, k, current_size)? {
            pack::PackResult::Packed(info) => return Ok(Some(PackedAtlas { damage_list, info })),
            pack::PackResult::NeedsDownscale => {
                if k >= MAX_DOWNSAMPLE_K {
                    return Err(AtlasError::builder(format!(
                        "Texture atlas does not fit within {}x{} even at downsample factor 2^{}",
                        max_atlas_size, max_atlas_size, k
                    )));
                }
                k += 1;
                current_size = (
                    current_size.0.saturating_mul(2).min(max_atlas_size),
                    current_size.1.saturating_mul(2).min(max_atlas_size),
                );
                damage_list = collect_damage(materials, k)?;
                if damage_list.is_empty() {
                    return Ok(None);
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
