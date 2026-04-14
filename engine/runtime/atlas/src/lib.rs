mod damage;
mod error;
mod pack;
mod skyline;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use damage::{collect_damage, DamageRect};
pub use error::{AtlasError, Result};
use image::ImageFormat;
use pack::pack_textures;

pub type PolygonUVs = Vec<[f64; 2]>;
pub type TextureUVs = Vec<PolygonUVs>;

#[derive(Debug, Clone)]
pub struct TextureMaterial {
    pub path: PathBuf,
    pub uvs: TextureUVs,
}

pub const MAX_DOWNSAMPLE_K: u32 = 13;

fn uv_bbox_in_pixel_space(uvs: &[[f64; 2]], tw: u32, th: u32) -> DamageRect {
    let (mn_u, mx_u, mn_v, mx_v) = uvs.iter().fold(
        (f64::MAX, f64::MIN, f64::MAX, f64::MIN),
        |(mn_u, mx_u, mn_v, mx_v), [u, v]| (mn_u.min(*u), mx_u.max(*u), mn_v.min(*v), mx_v.max(*v)),
    );
    let x = ((mn_u * tw as f64).floor().max(0.0) as u32).min(tw);
    let y = (((1.0 - mx_v) * th as f64).floor().max(0.0) as u32).min(th);
    let right = ((mx_u * tw as f64).ceil() as u32).min(tw);
    let bottom = (((1.0 - mn_v) * th as f64).ceil() as u32).min(th);
    DamageRect {
        x,
        y,
        w: right.saturating_sub(x),
        h: bottom.saturating_sub(y),
    }
}

fn remap_uv(
    u: f64,
    v: f64,
    tw: u32,
    th: u32,
    damage: DamageRect,
    frame: DamageRect,
    atlas_w: f64,
    atlas_h: f64,
    downsample: u32,
) -> [f64; 2] {
    let scale = downsample as f64;
    let px = u * tw as f64 - damage.x as f64;
    let py = (1.0 - v) * th as f64 - damage.y as f64;
    [
        (px / scale + frame.x as f64) / atlas_w,
        1.0 - (py / scale + frame.y as f64) / atlas_h,
    ]
}

/// Pack `materials` into an atlas and return remapped UVs.
/// `result[i]` is `Some(remapped_uvs)` if `materials[i]` was packed, `None` if excluded.
pub fn build_atlas(
    materials: &[TextureMaterial],
    atlas_dir: &Path,
    image_format: ImageFormat,
    ext: &str,
    max_atlas_size: u32,
) -> Result<Vec<Option<TextureUVs>>> {
    let mut k = 0;
    let mut damage_list = collect_damage(materials, k)?;
    if damage_list.is_empty() {
        return Ok(materials.iter().map(|_| None).collect());
    }

    let mut current_size = pack::estimate_atlas_size(&damage_list, k, max_atlas_size);
    let info = loop {
        match pack_textures(
            &damage_list,
            atlas_dir,
            image_format,
            ext,
            k,
            current_size,
            max_atlas_size,
        )? {
            pack::PackResult::Packed(info) => break info,
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
                    return Ok(materials.iter().map(|_| None).collect());
                }
            }
        }
    };

    let tex_dims: HashMap<String, (u32, u32)> = damage_list
        .iter()
        .map(|(p, td)| (p.to_string_lossy().into_owned(), (td.width, td.height)))
        .collect();

    Ok(materials
        .iter()
        .map(|mat| {
            let path_str = mat.path.to_string_lossy().into_owned();
            let frames = info.texture_frames.get(&path_str)?;
            let (tw, th) = tex_dims.get(&path_str).copied().unwrap_or((1, 1));

            Some(
                mat.uvs
                    .iter()
                    .map(|poly_uvs| {
                        let uv_bbox = uv_bbox_in_pixel_space(poly_uvs, tw, th);
                        let Some((damage, frame)) = frames
                            .iter()
                            .find(|(d, _)| d.overlaps(uv_bbox))
                            .map(|(d, f)| (*d, *f))
                        else {
                            return poly_uvs.clone();
                        };
                        poly_uvs
                            .iter()
                            .map(|&[u, v]| {
                                remap_uv(
                                    u,
                                    v,
                                    tw,
                                    th,
                                    damage,
                                    frame,
                                    info.width as f64,
                                    info.height as f64,
                                    info.downsample,
                                )
                            })
                            .collect()
                    })
                    .collect(),
            )
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
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

        let atlas_dir = temp_dir.path().join("atlas");
        std::fs::create_dir(&atlas_dir).unwrap();

        let remapped = build_atlas(&materials, &atlas_dir, ImageFormat::Png, "png", 8192).unwrap();
        assert_eq!(remapped.len(), 2);
        assert!(remapped.iter().all(|entry| entry.is_some()));
        assert!(atlas_dir.join("0.png").exists());
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

        let atlas_dir = temp_dir.path().join("atlas");
        std::fs::create_dir(&atlas_dir).unwrap();

        build_atlas(
            &materials,
            &atlas_dir,
            ImageFormat::Png,
            "png",
            max_atlas_size,
        )
        .unwrap();

        let atlas = image::open(atlas_dir.join("0.png")).unwrap();
        assert!(atlas.width() <= max_atlas_size);
        assert!(atlas.height() <= max_atlas_size);
    }
}
