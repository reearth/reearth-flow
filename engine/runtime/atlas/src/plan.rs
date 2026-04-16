use super::skyline::SkylinePacker;
use super::Rect;

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
                ceil_div(w, downsample).max(1),
                ceil_div(h, downsample).max(1),
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

/// Returns the minimum k such that `side / 2^k <= max`, or `MAX_DOWNSAMPLE_K + 1` if none exists.
fn needed_k(side: u32, max: u32) -> u32 {
    if side <= max {
        return 0;
    }
    let ratio = side.div_ceil(max);
    if ratio > (1u32 << super::MAX_DOWNSAMPLE_K) {
        return super::MAX_DOWNSAMPLE_K + 1;
    }
    ratio.next_power_of_two().trailing_zeros()
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
    // `virtual_w/h` is the estimated canvas in original (pre-downsampling) pixel space.
    // Doubling grows the canvas at k=0; once the larger dimension exceeds max_atlas_size,
    // needed_k increments to keep the physical atlas within bounds. Both k and canvas are derived.
    let (mut virtual_w, mut virtual_h) = estimate_atlas_size_from_dims(dims, 0);
    loop {
        let k = needed_k(virtual_w.max(virtual_h), max_atlas_size);
        if k > super::MAX_DOWNSAMPLE_K {
            break;
        }
        let canvas = (virtual_w.min(max_atlas_size), virtual_h.min(max_atlas_size));
        if let Some((used_w, used_h, placements)) = try_layout_rects(dims, k, canvas) {
            return Ok(super::LayoutPlan {
                atlas_width: used_w,
                atlas_height: used_h,
                downsample: 1u32 << k,
                placements,
            });
        }
        virtual_w = virtual_w.saturating_mul(2);
        virtual_h = virtual_h.saturating_mul(2);
    }
    Err(super::AtlasError::builder(format!(
        "Texture atlas does not fit within {}x{} even at downsample factor 2^{}",
        max_atlas_size,
        max_atlas_size,
        super::MAX_DOWNSAMPLE_K
    )))
}
