//! Clearing atlas texels that no polygon samples.

use image::{Rgba, RgbaImage};

use crate::{MultiPageAtlas, PageWrap, PolygonUVs};

/// How the UVs of one polygon in [`crate::TextureInput::uvs`] are laid out.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum UvLayout {
    /// The polygon's ring; masking keeps its convex hull.
    #[default]
    Ring,
    /// Three corners per triangle; masking keeps the triangles.
    Triangles,
}

/// Sets every texel of the clamped pages of `atlas` farther than `margin`
/// pixels (Chebyshev) from every placed polygon to opaque black. Repeating
/// pages are left untouched.
pub fn clear_unused_texels(atlas: &mut MultiPageAtlas, layout: UvLayout, margin: u32) {
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
            clear_unused(page, polygons, layout, margin);
        }
    }
}

/// Sets every texel of `page` farther than `margin` pixels (Chebyshev) from
/// `polygons` to opaque black. `polygons` hold page UVs laid out as `layout`
/// says; a ring polygon counts as its convex hull.
fn clear_unused<'a>(
    page: &mut RgbaImage,
    polygons: impl IntoIterator<Item = &'a PolygonUVs>,
    layout: UvLayout,
    margin: u32,
) {
    let (width, height) = (page.width() as usize, page.height() as usize);
    let mut covered = vec![false; width * height];
    for uvs in polygons {
        let points: Vec<[f64; 2]> = uvs
            .iter()
            .map(|[u, v]| [u * width as f64, v * height as f64])
            .collect();
        match layout {
            UvLayout::Ring => fill_convex(&mut covered, width, height, &convex_hull(points)),
            UvLayout::Triangles => {
                for triangle in points.chunks_exact(3) {
                    fill_convex(&mut covered, width, height, &convex_hull(triangle.to_vec()));
                }
            }
        }
    }
    let covered = dilate(&covered, width, height, margin as usize);
    for (texel, keep) in page.pixels_mut().zip(covered) {
        if !keep {
            *texel = Rgba([0, 0, 0, 255]);
        }
    }
}

/// Marks the texels whose centres lie in the convex polygon `hull`, plus the
/// texels its vertices fall in so slivers thinner than a texel still count.
fn fill_convex(covered: &mut [bool], width: usize, height: usize, hull: &[[f64; 2]]) {
    for &[x, y] in hull {
        let (tx, ty) = (x.floor() as isize, y.floor() as isize);
        if (0..width as isize).contains(&tx) && (0..height as isize).contains(&ty) {
            covered[ty as usize * width + tx as usize] = true;
        }
    }
    if hull.len() < 3 {
        return;
    }
    let min_y = hull.iter().map(|p| p[1]).fold(f64::MAX, f64::min);
    let max_y = hull.iter().map(|p| p[1]).fold(f64::MIN, f64::max);
    let first = (min_y - 0.5).ceil().max(0.0);
    let last = (max_y - 0.5).floor().min(height as f64 - 1.0);
    if last < first {
        return;
    }
    for row in first as usize..=last as usize {
        let y = row as f64 + 0.5;
        let (mut left, mut right) = (f64::MAX, f64::MIN);
        for (i, &[ax, ay]) in hull.iter().enumerate() {
            let [bx, by] = hull[(i + 1) % hull.len()];
            if (ay - y) * (by - y) > 0.0 {
                continue;
            }
            let (from, to) = if ay == by {
                (ax.min(bx), ax.max(bx))
            } else {
                let x = ax + (y - ay) * (bx - ax) / (by - ay);
                (x, x)
            };
            left = left.min(from);
            right = right.max(to);
        }
        let from = (left - 0.5).ceil().max(0.0);
        let to = (right - 0.5).floor().min(width as f64 - 1.0);
        if to >= from {
            covered[row * width + from as usize..=row * width + to as usize].fill(true);
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

/// Convex hull in counter-clockwise order, collinear points dropped.
fn convex_hull(mut points: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    points.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    points.dedup();
    if points.len() < 3 {
        return points;
    }
    let cross = |o: [f64; 2], a: [f64; 2], b: [f64; 2]| {
        (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    };
    let mut hull: Vec<[f64; 2]> = Vec::with_capacity(points.len() * 2);
    for pass in [points.clone(), points.into_iter().rev().collect()] {
        let start = hull.len();
        for p in pass {
            while hull.len() >= start + 2
                && cross(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0.0
            {
                hull.pop();
            }
            hull.push(p);
        }
        hull.pop();
    }
    hull
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clears_texels_beyond_the_margin() {
        let mut page = RgbaImage::from_pixel(10, 10, Rgba([200, 200, 200, 255]));
        let square = vec![[0.2, 0.2], [0.4, 0.2], [0.4, 0.4], [0.2, 0.4]];
        clear_unused(&mut page, [&square], UvLayout::Ring, 1);
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
        clear_unused(&mut page, [&triangle], UvLayout::Ring, 0);
        assert_eq!(page.get_pixel(2, 2).0, [9, 9, 9, 255]);
        assert_eq!(page.get_pixel(18, 18).0, [0, 0, 0, 255]);
    }

    #[test]
    fn triangle_layout_clears_the_notch_of_a_concave_polygon() {
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
        let mut rings = RgbaImage::from_pixel(20, 20, Rgba([9, 9, 9, 255]));
        clear_unused(&mut rings, [&l_shape], UvLayout::Ring, 0);
        assert_eq!(
            rings.get_pixel(14, 7).0,
            [9, 9, 9, 255],
            "the hull keeps the notch"
        );
        let mut triangles = RgbaImage::from_pixel(20, 20, Rgba([9, 9, 9, 255]));
        clear_unused(&mut triangles, [&l_shape], UvLayout::Triangles, 0);
        assert_eq!(triangles.get_pixel(14, 7).0, [0, 0, 0, 255]);
        assert_eq!(triangles.get_pixel(15, 15).0, [9, 9, 9, 255]);
    }
}
