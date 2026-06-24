use super::skyline::SkylinePacker;
use super::Rect;

pub(crate) fn estimate_atlas_size_from_dims(dims: &[(u32, u32)]) -> u32 {
    const EXTRUSION: u32 = 1;
    let total_area: u64 = dims
        .iter()
        .map(|&(w, h)| (w + 2 * EXTRUSION) as u64 * (h + 2 * EXTRUSION) as u64)
        .sum();
    let max_padded_w = dims
        .iter()
        .map(|&(w, _)| w + 2 * EXTRUSION)
        .max()
        .unwrap_or(0);
    let area_side = (total_area as f64).sqrt().ceil() as u32;
    max_padded_w.max(area_side).next_power_of_two().max(1)
}

/// Dry-run layout — no image I/O, no blitting.
/// Returns `Some((used_w, used_h, placements))` if all rects fit, `None` otherwise.
/// `placements[i]` is the atlas-space rect for `dims[i]`.
pub(crate) fn try_layout_rects(
    dims: &[(u32, u32)],
    k: u32,
    canvas: (u32, u32),
) -> Option<(u32, u32, Vec<Rect>)> {
    let downsample = 1u32 << k;
    let extrusion = 1u32;
    // Pair each rect with its original index before sorting.
    let mut indexed: Vec<(usize, u32, u32)> = dims
        .iter()
        .enumerate()
        .map(|(i, &(w, h))| {
            (
                i,
                w.div_ceil(downsample).max(1),
                h.div_ceil(downsample).max(1),
            )
        })
        .collect();
    indexed.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| b.1.cmp(&a.1)));
    let mut packer = SkylinePacker::new(canvas.0, canvas.1, extrusion);
    let mut placements_sorted: Vec<(usize, Rect)> = Vec::with_capacity(dims.len());
    for &(orig_idx, w, h) in &indexed {
        let frame = packer.pack(w, h)?;
        placements_sorted.push((
            orig_idx,
            Rect {
                x: frame.x,
                y: frame.y,
                w,
                h,
            },
        ));
    }
    placements_sorted.sort_by_key(|t| t.0);
    let placements = placements_sorted.into_iter().map(|(_, r)| r).collect();
    Some((packer.width(), packer.height(), placements))
}

/// Compute the layout for a set of textures given only their dimensions.
/// No image files are read; no pixels are blitted.
/// Useful for efficiency benchmarks and layout-only unit tests.
pub fn plan_layout(dims: &[(u32, u32)], max_atlas_size: u32) -> super::Result<super::LayoutPlan> {
    if dims.is_empty() {
        return Ok(super::LayoutPlan {
            atlas_width: 1,
            atlas_height: 1,
            downsample: 1,
            placements: vec![],
        });
    }
    let initial_w = estimate_atlas_size_from_dims(dims);
    for k in 0..=super::MAX_DOWNSAMPLE_K {
        let mut canvas_w = initial_w;
        loop {
            let w = canvas_w.min(max_atlas_size);
            if let Some((used_w, used_h, placements)) =
                try_layout_rects(dims, k, (w, max_atlas_size))
            {
                return Ok(super::LayoutPlan {
                    atlas_width: used_w,
                    atlas_height: used_h,
                    downsample: 1u32 << k,
                    placements,
                });
            }
            if canvas_w >= max_atlas_size {
                break;
            }
            canvas_w = canvas_w.saturating_mul(2);
        }
    }
    Err(super::AtlasError::builder(format!(
        "Texture atlas does not fit within {}x{} even at downsample factor 2^{}",
        max_atlas_size,
        max_atlas_size,
        super::MAX_DOWNSAMPLE_K
    )))
}
