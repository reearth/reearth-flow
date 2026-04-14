use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::damage::{DamageRect, TextureDamage};
use super::skyline::SkylinePacker;
use super::MAX_DOWNSAMPLE_K;
use image::imageops::FilterType;
use image::{GenericImage, ImageFormat, Rgba, RgbaImage};

pub struct AtlasInfo {
    pub texture_frames: HashMap<String, Vec<(DamageRect, DamageRect)>>,
    pub width: u32,
    pub height: u32,
    pub downsample: u32,
}

pub enum PackResult {
    Packed(AtlasInfo),
    NeedsDownscale,
}

struct Candidate<'a> {
    path: &'a Path,
    path_str: String,
    rect: DamageRect,
    key: String,
    w: u32,
    h: u32,
}

fn downsample_factor(k: u32) -> crate::Result<u32> {
    if k > MAX_DOWNSAMPLE_K {
        return Err(crate::AtlasError::builder(format!(
            "Unsupported atlas downsample exponent: {k}"
        )));
    }
    Ok(1u32 << k)
}

fn ceil_div(value: u32, divisor: u32) -> u32 {
    value.div_ceil(divisor)
}

pub(super) fn estimate_atlas_size(
    damage_list: &[(PathBuf, TextureDamage)],
    k: u32,
    max_atlas_size: u32,
) -> (u32, u32) {
    if damage_list.is_empty() {
        return (1, 1);
    }
    let downsample = 1u32 << k;
    let extrusion = 1;
    let total_area: u64 = damage_list
        .iter()
        .flat_map(|(_, td)| td.rects.iter())
        .map(|r| {
            let w = ceil_div(r.w, downsample) + 2 * extrusion;
            let h = ceil_div(r.h, downsample) + 2 * extrusion;
            w as u64 * h as u64
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
    let side = (total_area as f64).sqrt().ceil() as u32;
    (
        max_w.max(side).next_power_of_two().min(max_atlas_size),
        max_h.max(side).next_power_of_two().min(max_atlas_size),
    )
}

fn build_candidates<'a>(
    damage_list: &'a [(PathBuf, TextureDamage)],
    downsample: u32,
) -> Vec<Candidate<'a>> {
    let mut out = Vec::new();
    for (path, td) in damage_list {
        let path_str = path.to_string_lossy().into_owned();
        for (i, &rect) in td.rects.iter().enumerate() {
            out.push(Candidate {
                path,
                path_str: path_str.clone(),
                rect,
                key: format!("{}#{i}", path.to_string_lossy()),
                w: ceil_div(rect.w, downsample).max(1),
                h: ceil_div(rect.h, downsample).max(1),
            });
        }
    }
    out.sort_by(|a, b| b.h.cmp(&a.h).then_with(|| b.w.cmp(&a.w)));
    out
}

fn fill_frame_extrusion(atlas: &mut RgbaImage, frame: DamageRect, extrusion: u32) {
    if extrusion == 0 || frame.w == 0 || frame.h == 0 {
        return;
    }
    let left = frame.x;
    let top = frame.y;
    let right = frame.right() - 1;
    let bottom = frame.bottom() - 1;
    let fill_left = left.saturating_sub(extrusion);
    let fill_top = top.saturating_sub(extrusion);
    let fill_right = right.saturating_add(extrusion).min(atlas.width() - 1);
    let fill_bottom = bottom.saturating_add(extrusion).min(atlas.height() - 1);

    for y in top..=bottom {
        let a = *atlas.get_pixel(left, y);
        let b = *atlas.get_pixel(right, y);
        for x in fill_left..left {
            atlas.put_pixel(x, y, a);
        }
        for x in right + 1..=fill_right {
            atlas.put_pixel(x, y, b);
        }
    }
    for x in fill_left..=fill_right {
        let a = *atlas.get_pixel(x, top);
        let b = *atlas.get_pixel(x, bottom);
        for y in fill_top..top {
            atlas.put_pixel(x, y, a);
        }
        for y in bottom + 1..=fill_bottom {
            atlas.put_pixel(x, y, b);
        }
    }
}

pub fn pack_textures(
    damage_list: &[(PathBuf, TextureDamage)],
    atlas_dir: &Path,
    image_format: ImageFormat,
    ext: &str,
    k: u32,
    current_size: (u32, u32),
) -> crate::Result<PackResult> {
    let downsample = downsample_factor(k)?;
    let extrusion = 1;
    let candidates = build_candidates(damage_list, downsample);
    let mut layout = SkylinePacker::new(current_size.0, current_size.1, extrusion);
    let mut frames = HashMap::new();
    for c in &candidates {
        let Some(frame) = layout.pack(c.w, c.h) else {
            return Ok(PackResult::NeedsDownscale);
        };
        frames.insert(c.key.clone(), frame);
    }

    let mut atlas = RgbaImage::from_pixel(layout.width(), layout.height(), Rgba([0, 0, 0, 0]));
    let mut sources = HashMap::new();
    let mut texture_frames: HashMap<String, Vec<(DamageRect, DamageRect)>> = HashMap::new();

    for c in candidates {
        let source = match sources.entry(c.path) {
            std::collections::hash_map::Entry::Occupied(e) => e.into_mut(),
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(image::open(c.path).map_err(|e| {
                    crate::AtlasError::builder(format!(
                        "Failed to open texture '{}': {e}",
                        c.path.display()
                    ))
                })?)
            }
        };
        let mut crop = source
            .crop_imm(c.rect.x, c.rect.y, c.rect.w, c.rect.h)
            .to_rgba8();
        if downsample > 1 {
            crop = image::imageops::resize(&crop, c.w, c.h, FilterType::Triangle);
        }
        let frame = frames[&c.key];
        atlas.copy_from(&crop, frame.x, frame.y).map_err(|_| {
            crate::AtlasError::builder("Internal bug: failed to copy texture into atlas")
        })?;
        fill_frame_extrusion(&mut atlas, frame, extrusion);
        texture_frames
            .entry(c.path_str)
            .or_default()
            .push((c.rect, frame));
    }

    atlas
        .save_with_format(atlas_dir.join("0").with_extension(ext), image_format)
        .map_err(|e| crate::AtlasError::builder(format!("Failed to save atlas: {e}")))?;

    Ok(PackResult::Packed(AtlasInfo {
        texture_frames,
        width: atlas.width(),
        height: atlas.height(),
        downsample,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let (w, h) = estimate_atlas_size(&dims, 0, 8192);
        assert_eq!((w, h), (256, 256));
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
        let (w, h) = estimate_atlas_size(&dims, 0, 8192);
        assert_eq!((w, h), (512, 512));
    }

    #[test]
    fn test_estimate_empty() {
        assert_eq!(estimate_atlas_size(&[], 0, 8192), (1, 1));
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
        assert_eq!(estimate_atlas_size(&dims, 0, 8192), (1024, 1024));
    }

    #[test]
    fn test_fill_frame_extrusion_replicates_edge_pixels() {
        let mut atlas = RgbaImage::new(4, 4);
        atlas.put_pixel(1, 1, Rgba([10, 11, 12, 255]));
        atlas.put_pixel(2, 1, Rgba([20, 21, 22, 255]));
        atlas.put_pixel(1, 2, Rgba([30, 31, 32, 255]));
        atlas.put_pixel(2, 2, Rgba([40, 41, 42, 255]));
        fill_frame_extrusion(
            &mut atlas,
            DamageRect {
                x: 1,
                y: 1,
                w: 2,
                h: 2,
            },
            1,
        );
        assert_eq!(*atlas.get_pixel(0, 0), Rgba([10, 11, 12, 255]));
        assert_eq!(*atlas.get_pixel(3, 0), Rgba([20, 21, 22, 255]));
        assert_eq!(*atlas.get_pixel(0, 3), Rgba([30, 31, 32, 255]));
        assert_eq!(*atlas.get_pixel(3, 3), Rgba([40, 41, 42, 255]));
    }
}
