use std::collections::HashMap;
use std::path::PathBuf;

use super::damage::TextureDamage;
use super::Rect;
use image::imageops::FilterType;
use image::{GenericImage, Rgba, RgbaImage};

pub(super) type TextureFrames = HashMap<String, Vec<(Rect, Rect)>>;

fn fill_frame_extrusion(atlas: &mut RgbaImage, frame: Rect, extrusion: u32) {
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

/// Blit all damage regions into an atlas using pre-computed placements from `plan_layout`.
/// `placements[i]` is the atlas-space rect for the i-th rect in the flattened damage list.
pub(super) fn blit(
    damage_list: &[(PathBuf, TextureDamage)],
    atlas_size: (u32, u32),
    placements: &[Rect],
) -> crate::Result<(RgbaImage, TextureFrames)> {
    let extrusion = 1u32;
    let mut atlas = RgbaImage::from_pixel(atlas_size.0, atlas_size.1, Rgba([0, 0, 0, 0]));
    let mut sources = HashMap::new();
    let mut texture_frames: HashMap<String, Vec<Option<(Rect, Rect)>>> = damage_list
        .iter()
        .map(|(path, td)| {
            (
                path.to_string_lossy().into_owned(),
                vec![None; td.rects.len()],
            )
        })
        .collect();

    let mut flat_idx = 0usize;
    for (path, td) in damage_list {
        let path_str = path.to_string_lossy().into_owned();
        let frames_slot = texture_frames.get_mut(&path_str).expect("pre-built above");
        for (region_index, &src_rect) in td.rects.iter().enumerate() {
            let placement = placements[flat_idx];
            flat_idx += 1;

            let source = match sources.entry(path.as_path()) {
                std::collections::hash_map::Entry::Occupied(e) => e.into_mut(),
                std::collections::hash_map::Entry::Vacant(e) => {
                    e.insert(image::open(path).map_err(|e| {
                        crate::AtlasError::builder(format!(
                            "Failed to open texture '{}': {e}",
                            path.display()
                        ))
                    })?)
                }
            };
            let mut crop = source
                .crop_imm(src_rect.x, src_rect.y, src_rect.w, src_rect.h)
                .to_rgba8();
            if (placement.w, placement.h) != (src_rect.w, src_rect.h) {
                crop =
                    image::imageops::resize(&crop, placement.w, placement.h, FilterType::Triangle);
            }
            atlas
                .copy_from(&crop, placement.x, placement.y)
                .map_err(|_| {
                    crate::AtlasError::builder("Internal bug: failed to copy texture into atlas")
                })?;
            fill_frame_extrusion(&mut atlas, placement, extrusion);
            frames_slot[region_index] = Some((src_rect, placement));
        }
    }

    let texture_frames = texture_frames
        .into_iter()
        .map(|(path, frames)| {
            let frames = frames
                .into_iter()
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| crate::AtlasError::builder("Internal bug: missing atlas frame"))?;
            Ok((path, frames))
        })
        .collect::<crate::Result<HashMap<_, _>>>()?;

    Ok((atlas, texture_frames))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_frame_extrusion_replicates_edge_pixels() {
        let mut atlas = RgbaImage::new(4, 4);
        atlas.put_pixel(1, 1, Rgba([10, 11, 12, 255]));
        atlas.put_pixel(2, 1, Rgba([20, 21, 22, 255]));
        atlas.put_pixel(1, 2, Rgba([30, 31, 32, 255]));
        atlas.put_pixel(2, 2, Rgba([40, 41, 42, 255]));
        fill_frame_extrusion(
            &mut atlas,
            Rect {
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
