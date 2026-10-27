//! Clearing atlas texels that no polygon samples.

use image::{Rgba, RgbaImage};

use crate::{MultiPageAtlas, PageWrap, PolygonUVs};

/// Sets every texel of the clamped pages of `atlas` farther than `margin`
/// pixels (Chebyshev) from every placed triangle to opaque black, keeping
/// whole `block` x `block` tiles (aligned to the page origin) that hold any
/// kept texel. Repeating pages are left untouched. Each placement's UVs must
/// list three corners per triangle; `margin` 0 is taken as 1 and `block` 0
/// as 1.
pub fn clear_unused_texels(atlas: &mut MultiPageAtlas, margin: u32, block: u32) {
    let mut polygons_by_page: Vec<Vec<&PolygonUVs>> = vec![Vec::new(); atlas.pages.len()];
    for placement in atlas.remapped.iter().flatten().flatten() {
        polygons_by_page[placement.page].push(&placement.uvs);
    }
    for ((page, polygons), wrap) in atlas
        .pages
        .iter_mut()
        .zip(polygons_by_page)
        .zip(&atlas.wrap)
    {
        if *wrap == PageWrap::Clamp {
            clear_unused(page, polygons, margin.max(1), block);
        }
    }
}

/// Sets every texel of `page` farther than `margin` pixels (Chebyshev) from
/// the triangles in `polygons` to opaque black. `polygons` hold page UVs, three
/// corners per triangle. Whole `block` x `block` tiles holding any kept texel
/// are kept.
fn clear_unused<'a>(
    page: &mut RgbaImage,
    polygons: impl IntoIterator<Item = &'a PolygonUVs>,
    margin: u32,
    block: u32,
) {
    let (width, height) = (page.width() as usize, page.height() as usize);
    let mut covered = vec![false; width * height];
    for uvs in polygons {
        let points: Vec<[f64; 2]> = uvs
            .iter()
            .map(|[u, v]| [u * width as f64, v * height as f64])
            .collect();
        for triangle in points.as_chunks::<3>().0 {
            fill_triangle(&mut covered, width, height, triangle);
        }
    }
    let mut covered = dilate(&covered, width, height, margin as usize);
    snap_to_blocks(&mut covered, width, height, block.max(1) as usize);
    for (texel, keep) in page.pixels_mut().zip(covered) {
        if !keep {
            *texel = Rgba([0, 0, 0, 255]);
        }
    }
}

/// Marks every texel whose square overlaps `triangle`.
fn fill_triangle(covered: &mut [bool], width: usize, height: usize, triangle: &[[f64; 2]]) {
    let min_y = triangle.iter().map(|p| p[1]).fold(f64::MAX, f64::min);
    let max_y = triangle.iter().map(|p| p[1]).fold(f64::MIN, f64::max);
    let first = min_y.floor().max(0.0);
    let last = (max_y.ceil() - 1.0)
        .max(min_y.floor())
        .min(height as f64 - 1.0);
    if last < first {
        return;
    }
    for row in first as usize..=last as usize {
        let (top, bottom) = (row as f64, row as f64 + 1.0);
        let (mut left, mut right) = (f64::MAX, f64::MIN);
        for (i, &[ax, ay]) in triangle.iter().enumerate() {
            let [bx, by] = triangle[(i + 1) % triangle.len()];
            let (lo, hi) = if ay == by {
                if ay < top || ay > bottom {
                    continue;
                }
                (0.0, 1.0)
            } else {
                let (t0, t1) = ((top - ay) / (by - ay), (bottom - ay) / (by - ay));
                (t0.min(t1).max(0.0), t0.max(t1).min(1.0))
            };
            if lo > hi {
                continue;
            }
            for t in [lo, hi] {
                let x = ax + t * (bx - ax);
                left = left.min(x);
                right = right.max(x);
            }
        }
        if left > right {
            continue;
        }
        let from = left.floor().max(0.0);
        let to = (right.ceil() - 1.0)
            .max(left.floor())
            .min(width as f64 - 1.0);
        if to >= from {
            covered[row * width + from as usize..=row * width + to as usize].fill(true);
        }
    }
}

/// Marks every texel of each `block` x `block` tile, aligned to the origin,
/// that holds a marked texel.
fn snap_to_blocks(mask: &mut [bool], width: usize, height: usize, block: usize) {
    if block == 1 {
        return;
    }
    for by in (0..height).step_by(block) {
        let rows = by..(by + block).min(height);
        for bx in (0..width).step_by(block) {
            let cols = bx..(bx + block).min(width);
            if rows
                .clone()
                .any(|y| mask[y * width + cols.start..y * width + cols.end].contains(&true))
            {
                for y in rows.clone() {
                    mask[y * width + cols.start..y * width + cols.end].fill(true);
                }
            }
        }
    }
}

/// Grows the marked texels by `radius` in every direction (a square kernel).
fn dilate(mask: &[bool], width: usize, height: usize, radius: usize) -> Vec<bool> {
    if radius == 0 {
        return mask.to_vec();
    }
    let grow = |line: &[bool]| -> Vec<bool> {
        let mut prefix = vec![0u32; line.len() + 1];
        for (i, &m) in line.iter().enumerate() {
            prefix[i + 1] = prefix[i] + m as u32;
        }
        (0..line.len())
            .map(|i| {
                let lo = i.saturating_sub(radius);
                let hi = (i + radius + 1).min(line.len());
                prefix[hi] > prefix[lo]
            })
            .collect()
    };
    let mut rows = Vec::with_capacity(mask.len());
    for row in mask.chunks(width) {
        rows.extend(grow(row));
    }
    let mut out = vec![false; mask.len()];
    let mut column = vec![false; height];
    for x in 0..width {
        for y in 0..height {
            column[y] = rows[y * width + x];
        }
        for (y, keep) in grow(&column).into_iter().enumerate() {
            out[y * width + x] = keep;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two triangles covering the axis-aligned square from `lo` to `hi`.
    fn square(lo: f64, hi: f64) -> PolygonUVs {
        vec![[lo, lo], [hi, lo], [hi, hi], [lo, lo], [hi, hi], [lo, hi]]
    }

    #[test]
    fn clears_texels_beyond_the_margin() {
        let mut page = RgbaImage::from_pixel(10, 10, Rgba([200, 200, 200, 255]));
        clear_unused(&mut page, [&square(0.2, 0.4)], 1, 1);
        assert_eq!(page.get_pixel(3, 3).0, [200, 200, 200, 255]);
        assert_eq!(
            page.get_pixel(1, 3).0,
            [200, 200, 200, 255],
            "within the margin"
        );
        assert_eq!(page.get_pixel(0, 3).0, [0, 0, 0, 255]);
        assert_eq!(page.get_pixel(8, 8).0, [0, 0, 0, 255]);
    }

    #[test]
    fn keeps_a_triangle_and_clears_beside_it() {
        let mut page = RgbaImage::from_pixel(20, 20, Rgba([9, 9, 9, 255]));
        let triangle = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        clear_unused(&mut page, [&triangle], 0, 1);
        assert_eq!(page.get_pixel(2, 2).0, [9, 9, 9, 255]);
        assert_eq!(page.get_pixel(18, 18).0, [0, 0, 0, 255]);
    }

    #[test]
    fn keeps_whole_blocks_that_hold_a_kept_texel() {
        let mut page = RgbaImage::from_pixel(12, 12, Rgba([9, 9, 9, 255]));
        clear_unused(&mut page, [&square(0.4, 0.45)], 0, 4);
        for (x, y, texel) in page.enumerate_pixels() {
            let kept = (4..8).contains(&x) && (4..8).contains(&y);
            assert_eq!(texel.0[0] == 9, kept, "texel ({x}, {y})");
        }
    }

    #[test]
    fn keeps_every_texel_a_sliver_crosses() {
        let sliver = vec![[0.0, 0.3 / 8.0], [1.0, 8.3 / 8.0], [1.0, 8.25 / 8.0]];
        let mut page = RgbaImage::from_pixel(8, 8, Rgba([9, 9, 9, 255]));
        clear_unused(&mut page, [&sliver], 0, 1);
        for (x, y, texel) in page.enumerate_pixels() {
            let kept = y == x || y == x + 1;
            assert_eq!(texel.0[0] == 9, kept, "texel ({x}, {y})");
        }
    }

    #[test]
    fn keeps_a_sliver_between_texel_centres() {
        let sliver = vec![[0.0, 0.1], [1.0, 0.1], [1.0, 0.12]];
        let mut page = RgbaImage::from_pixel(10, 10, Rgba([9, 9, 9, 255]));
        clear_unused(&mut page, [&sliver], 0, 1);
        for (x, y, texel) in page.enumerate_pixels() {
            assert_eq!(texel.0[0] == 9, y == 1, "texel ({x}, {y})");
        }
    }

    #[test]
    fn clears_the_notch_of_a_concave_polygon() {
        let l_shape = vec![
            [0.0, 0.0],
            [0.5, 0.0],
            [0.5, 1.0],
            [0.0, 0.0],
            [0.5, 1.0],
            [0.0, 1.0],
            [0.5, 0.5],
            [1.0, 0.5],
            [1.0, 1.0],
            [0.5, 0.5],
            [1.0, 1.0],
            [0.5, 1.0],
        ];
        let mut page = RgbaImage::from_pixel(20, 20, Rgba([9, 9, 9, 255]));
        clear_unused(&mut page, [&l_shape], 0, 1);
        assert_eq!(page.get_pixel(14, 7).0, [0, 0, 0, 255]);
        assert_eq!(page.get_pixel(15, 15).0, [9, 9, 9, 255]);
    }
}
