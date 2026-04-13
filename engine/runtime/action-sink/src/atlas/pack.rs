use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::damage::{DamageRect, TextureDamage};
use super::{MAX_ATLAS_SIZE, MAX_DOWNSAMPLE_K};
use image::imageops::FilterType;
use image::{ImageFormat, RgbaImage};
use texture_packer::exporter::ImageExporter;
use texture_packer::texture::Texture;
use texture_packer::{TexturePacker, TexturePackerConfig};

pub struct AtlasInfo {
    /// Maps texture path string → list of (damage_rect, atlas_frame_rect).
    pub texture_frames: HashMap<String, Vec<(DamageRect, texture_packer::Rect)>>,
    pub width: u32,
    pub height: u32,
    pub downsample: u32,
}

pub enum PackResult {
    Packed(AtlasInfo),
    NeedsDownscale,
}

fn downsample_factor(k: u32) -> crate::errors::Result<u32> {
    if k > MAX_DOWNSAMPLE_K {
        return Err(crate::errors::SinkError::atlas_builder(format!(
            "Unsupported atlas downsample exponent: {k}"
        )));
    }
    Ok(1u32 << k)
}

fn ceil_div(value: u32, divisor: u32) -> u32 {
    value.div_ceil(divisor)
}

pub(super) fn estimate_atlas_size(damage_list: &[(PathBuf, TextureDamage)], k: u32) -> (u32, u32) {
    if damage_list.is_empty() {
        return (1, 1);
    }
    let downsample = 1u32 << k;
    let extrusion = downsample;
    let total_area: u64 = damage_list
        .iter()
        .flat_map(|(_, td)| td.rects.iter())
        .map(|r| {
            let packed_w = ceil_div(r.w, downsample) + 2 * extrusion;
            let packed_h = ceil_div(r.h, downsample) + 2 * extrusion;
            packed_w as u64 * packed_h as u64
        })
        .sum();
    let max_w = damage_list
        .iter()
        .flat_map(|(_, td)| td.rects.iter())
        .map(|r| ceil_div(r.w, downsample) + 2 * extrusion)
        .max()
        .unwrap_or(0);
    let max_h = damage_list
        .iter()
        .flat_map(|(_, td)| td.rects.iter())
        .map(|r| ceil_div(r.h, downsample) + 2 * extrusion)
        .max()
        .unwrap_or(0);
    let area_side = (total_area as f64).sqrt().ceil() as u32;
    let width = max_w.max(area_side).next_power_of_two().min(MAX_ATLAS_SIZE);
    let height = max_h.max(area_side).next_power_of_two().min(MAX_ATLAS_SIZE);
    (width, height)
}

fn make_pack_key(path: &Path, rect_idx: usize) -> String {
    format!("{}#{}", path.to_string_lossy(), rect_idx)
}

pub fn pack_textures(
    damage_list: &[(PathBuf, TextureDamage)],
    atlas_dir: &Path,
    image_format: ImageFormat,
    ext: &str,
    k: u32,
    current_size: (u32, u32),
) -> crate::errors::Result<PackResult> {
    if damage_list.is_empty() {
        return Ok(PackResult::NeedsDownscale);
    }
    let downsample = downsample_factor(k)?;
    let extrusion = downsample;
    let (atlas_w, atlas_h) = current_size;

    let config = TexturePackerConfig {
        max_width: atlas_w,
        max_height: atlas_h,
        allow_rotation: false,
        trim: false,
        texture_extrusion: extrusion,
        force_max_dimensions: false,
        ..Default::default()
    };
    let mut packer: TexturePacker<RgbaImage, String> = TexturePacker::new_skyline(config);

    let mut key_to_info: HashMap<String, (String, DamageRect)> = HashMap::new();

    for (path, td) in damage_list {
        let source = image::open(path).map_err(|e| {
            crate::errors::SinkError::atlas_builder(format!(
                "Failed to open texture '{}': {e}",
                path.display()
            ))
        })?;
        let path_str = path.to_string_lossy().into_owned();

        for (i, &rect) in td.rects.iter().enumerate() {
            let crop = source.crop_imm(rect.x, rect.y, rect.w, rect.h).to_rgba8();
            let crop = if downsample > 1 {
                image::imageops::resize(
                    &crop,
                    ceil_div(rect.w, downsample).max(1),
                    ceil_div(rect.h, downsample).max(1),
                    FilterType::Triangle,
                )
            } else {
                crop
            };
            let key = make_pack_key(path, i);
            if packer.pack_own(key.clone(), crop).is_err() {
                return Ok(PackResult::NeedsDownscale);
            }
            key_to_info.insert(key, (path_str.clone(), rect));
        }
    }

    let actual_w = packer.width();
    let actual_h = packer.height();

    let atlas_image = ImageExporter::export(&packer, None)
        .map_err(|e| crate::errors::SinkError::atlas_builder(e))?;

    let atlas_path = atlas_dir.join("0").with_extension(ext);
    atlas_image
        .save_with_format(&atlas_path, image_format)
        .map_err(|e| {
            crate::errors::SinkError::atlas_builder(format!("Failed to save atlas: {e}"))
        })?;

    let mut texture_frames: HashMap<String, Vec<(DamageRect, texture_packer::Rect)>> =
        HashMap::new();
    for (key, frame) in packer.get_frames() {
        if let Some((path_str, damage_rect)) = key_to_info.get(key) {
            texture_frames
                .entry(path_str.clone())
                .or_default()
                .push((*damage_rect, frame.frame));
        }
    }

    Ok(PackResult::Packed(AtlasInfo {
        texture_frames,
        width: actual_w,
        height: actual_h,
        downsample,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_estimate_atlas_size() {
        let dims = vec![
            (
                PathBuf::from("a"),
                TextureDamage {
                    width: 100,
                    height: 80,
                    rects: vec![DamageRect {
                        x: 0,
                        y: 0,
                        w: 100,
                        h: 80,
                    }],
                },
            ),
            (
                PathBuf::from("b"),
                TextureDamage {
                    width: 100,
                    height: 80,
                    rects: vec![DamageRect {
                        x: 0,
                        y: 0,
                        w: 100,
                        h: 80,
                    }],
                },
            ),
            (
                PathBuf::from("c"),
                TextureDamage {
                    width: 50,
                    height: 120,
                    rects: vec![DamageRect {
                        x: 0,
                        y: 0,
                        w: 50,
                        h: 120,
                    }],
                },
            ),
        ];
        let (w, h) = estimate_atlas_size(&dims, 0);
        assert_eq!(w, 256);
        assert_eq!(h, 256);
    }

    #[test]
    fn test_estimate_two_large_square() {
        let dims = vec![
            (
                PathBuf::from("a"),
                TextureDamage {
                    width: 256,
                    height: 256,
                    rects: vec![DamageRect {
                        x: 0,
                        y: 0,
                        w: 256,
                        h: 256,
                    }],
                },
            ),
            (
                PathBuf::from("b"),
                TextureDamage {
                    width: 256,
                    height: 256,
                    rects: vec![DamageRect {
                        x: 0,
                        y: 0,
                        w: 256,
                        h: 256,
                    }],
                },
            ),
        ];
        let (w, h) = estimate_atlas_size(&dims, 0);
        assert_eq!(w, 512);
        assert_eq!(h, 512);
    }

    #[test]
    fn test_estimate_empty() {
        let (w, h) = estimate_atlas_size(&[], 0);
        assert_eq!((w, h), (1, 1));
    }

    #[test]
    fn test_estimate_single() {
        let dims = vec![(
            PathBuf::from("a"),
            TextureDamage {
                width: 512,
                height: 512,
                rects: vec![DamageRect {
                    x: 0,
                    y: 0,
                    w: 512,
                    h: 512,
                }],
            },
        )];
        let (w, h) = estimate_atlas_size(&dims, 0);
        assert_eq!(w, 1024);
        assert_eq!(h, 1024);
    }
}
