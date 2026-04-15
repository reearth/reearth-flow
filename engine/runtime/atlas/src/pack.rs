use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::damage::{DamageRect, TextureDamage};
use super::skyline::SkylinePacker;
use image::imageops::FilterType;
use image::{GenericImage, Rgba, RgbaImage};

pub struct AtlasInfo {
    pub atlas: RgbaImage,
    pub texture_frames: HashMap<String, Vec<(DamageRect, DamageRect)>>,
    pub width: u32,
    pub height: u32,
    pub downsample: u32,
}

struct Candidate<'a> {
    path: &'a Path,
    path_str: String,
    region_index: usize,
    rect: DamageRect,
    key: String,
    w: u32,
    h: u32,
}

fn ceil_div(value: u32, divisor: u32) -> u32 {
    value.div_ceil(divisor)
}

pub(crate) fn estimate_atlas_size_from_dims(dims: &[(u32, u32)], k: u32) -> (u32, u32) {
    if dims.is_empty() {
        return (1, 1);
    }
    let downsample = 1u32 << k;
    let extrusion = 1u32;
    let total_area: u64 = dims
        .iter()
        .map(|&(w, h)| {
            let pw = ceil_div(w, downsample) + 2 * extrusion;
            let ph = ceil_div(h, downsample) + 2 * extrusion;
            pw as u64 * ph as u64
        })
        .sum();
    let max_w = dims
        .iter()
        .map(|&(w, _)| ceil_div(w, downsample) + 2 * extrusion)
        .max()
        .unwrap_or(0);
    let max_h = dims
        .iter()
        .map(|&(_, h)| ceil_div(h, downsample) + 2 * extrusion)
        .max()
        .unwrap_or(0);
    let side = (total_area as f64).sqrt().ceil() as u32;
    (
        max_w.max(side).next_power_of_two(),
        max_h.max(side).next_power_of_two(),
    )
}

/// Dry-run layout — no image I/O, no blitting.
/// Returns `Some((used_w, used_h, placements))` if all rects fit, `None` otherwise.
/// `placements[i]` is the atlas-space top-left `(x, y)` of the content rect for `dims[i]`.
pub(crate) fn try_layout_rects(
    dims: &[(u32, u32)],
    k: u32,
    canvas: (u32, u32),
) -> Option<(u32, u32, Vec<(u32, u32)>)> {
    let downsample = 1u32 << k;
    let extrusion = 1u32;
    // Pair each rect with its original index before sorting.
    let mut indexed: Vec<(usize, u32, u32)> = dims
        .iter()
        .enumerate()
        .map(|(i, &(w, h))| {
            (
                i,
                ceil_div(w, downsample).max(1),
                ceil_div(h, downsample).max(1),
            )
        })
        .collect();
    indexed.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| b.1.cmp(&a.1)));
    let mut packer = SkylinePacker::new(canvas.0, canvas.1, extrusion);
    let mut placements_sorted: Vec<(usize, u32, u32)> = Vec::with_capacity(dims.len());
    for &(orig_idx, w, h) in &indexed {
        let frame = packer.pack(w, h)?;
        placements_sorted.push((orig_idx, frame.x, frame.y));
    }
    placements_sorted.sort_by_key(|&(i, _, _)| i);
    let placements = placements_sorted
        .into_iter()
        .map(|(_, x, y)| (x, y))
        .collect();
    Some((packer.width(), packer.height(), placements))
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
                region_index: i,
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

/// Blit all damage regions into an atlas using pre-computed placements from `plan_layout`.
/// `placements[i]` is the atlas-space `(x, y)` for the i-th rect in the flattened damage list.
pub(super) fn blit(
    damage_list: &[(PathBuf, TextureDamage)],
    downsample: u32,
    atlas_size: (u32, u32),
    placements: &[(u32, u32)],
) -> crate::Result<AtlasInfo> {
    let extrusion = 1u32;
    let candidates = build_candidates(damage_list, downsample);
    // Build frames from placements (indexed by candidate key, same flat order as damage_list).
    let mut flat_idx = 0usize;
    let mut frames: HashMap<String, DamageRect> = HashMap::with_capacity(candidates.len());
    for (path, td) in damage_list {
        for (region_index, &rect) in td.rects.iter().enumerate() {
            let w = ceil_div(rect.w, downsample).max(1);
            let h = ceil_div(rect.h, downsample).max(1);
            let (x, y) = placements[flat_idx];
            let key = format!("{}#{region_index}", path.to_string_lossy());
            frames.insert(key, DamageRect { x, y, w, h });
            flat_idx += 1;
        }
    }

    let mut atlas = RgbaImage::from_pixel(atlas_size.0, atlas_size.1, Rgba([0, 0, 0, 0]));
    let mut sources = HashMap::new();
    let mut texture_frames: HashMap<String, Vec<Option<(DamageRect, DamageRect)>>> = damage_list
        .iter()
        .map(|(path, td)| {
            (
                path.to_string_lossy().into_owned(),
                vec![None; td.rects.len()],
            )
        })
        .collect();

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
        texture_frames.entry(c.path_str).or_default()[c.region_index] = Some((c.rect, frame));
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

    let width = atlas.width();
    let height = atlas.height();

    Ok(AtlasInfo {
        atlas,
        texture_frames,
        width,
        height,
        downsample,
    })
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
