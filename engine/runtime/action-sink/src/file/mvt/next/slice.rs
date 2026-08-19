use std::collections::HashMap;

use tinymvt::webmercator::lnglat_to_web_mercator;

use super::extract::Leaf;
use crate::file::mvt::tiling::TileContent;

pub(super) type Point = [f64; 2];
pub(super) type TileKey = (u8, u32, u32);

// 5px of margin on a standard 256px-wide web-map tile, as a fraction of tile width.
const MARGIN_WIDTH: f64 = 5.0 / 256.0;

pub(super) struct PolygonPart {
    pub(super) exterior: Vec<Point>,
    pub(super) holes: Vec<Vec<Point>>,
}

pub(super) enum TiledGeom {
    Polygon(Vec<PolygonPart>),
    /// Separate line paths landing in the same tile.
    LineString(Vec<Vec<Point>>),
    Point(Vec<Point>),
}

pub(super) struct TiledLeaf {
    pub(super) key: TileKey,
    pub(super) geom: TiledGeom,
}

pub(super) fn slice_leaves(
    leaves: Vec<Leaf>,
    min_z: u8,
    max_z: u8,
    extent: u32,
) -> (TileContent, Vec<TiledLeaf>) {
    let mut tiled_polys: HashMap<TileKey, Vec<PolygonPart>> = HashMap::new();
    let mut tiled_lines: HashMap<TileKey, Vec<Vec<Point>>> = HashMap::new();
    let mut tiled_points: HashMap<TileKey, Vec<Point>> = HashMap::new();
    let mut content = TileContent::default();

    for leaf in leaves {
        match leaf {
            Leaf::Polygon(rings) => {
                let mercator: Vec<Vec<Point>> = rings
                    .iter()
                    .map(|ring| project_ring(ring, &mut content))
                    .collect();
                let (exterior, holes) = normalize_winding(mercator);
                let y_bounds = ring_bounds(&exterior, 1);
                let x_bounds = ring_bounds(&exterior, 0);
                let diagonal = (x_bounds.1 - x_bounds.0).hypot(y_bounds.1 - y_bounds.0);

                for zoom in min_z..=max_z {
                    // early subpixel feature skip for efficient slicing
                    if diagonal * (1u64 << zoom) as f64 * (extent as f64) < 1.0 {
                        continue;
                    }
                    clip_polygon(&exterior, &holes, y_bounds, zoom, &mut tiled_polys);
                }
            }
            Leaf::LineString(coords) => {
                let mercator = project_ring(&coords, &mut content);
                let y_bounds = ring_bounds(&mercator, 1);

                for zoom in min_z..=max_z {
                    clip_line_string(&mercator, y_bounds, zoom, &mut tiled_lines);
                }
            }
            Leaf::Point(point) => {
                let [mx, my] = project_ring(&[point], &mut content)[0];
                for zoom in min_z..=max_z {
                    let z_scale = (1u64 << zoom) as f64;
                    let xi = (mx * z_scale).floor() as i64;
                    let yi = (my * z_scale).floor() as i64;
                    let key = tile_key(zoom, xi, yi);
                    let tx = mx * z_scale - xi as f64;
                    let ty = my * z_scale - yi as f64;
                    tiled_points.entry(key).or_default().push([tx, ty]);
                }
            }
        }
    }

    let mut tiled = Vec::new();
    tiled.extend(
        tiled_polys
            .into_iter()
            .filter(|(_, parts)| !parts.is_empty())
            .map(|(key, parts)| TiledLeaf {
                key,
                geom: TiledGeom::Polygon(parts),
            }),
    );
    tiled.extend(
        tiled_lines
            .into_iter()
            .filter(|(_, lines)| !lines.is_empty())
            .map(|(key, lines)| TiledLeaf {
                key,
                geom: TiledGeom::LineString(lines),
            }),
    );
    tiled.extend(
        tiled_points
            .into_iter()
            .filter(|(_, points)| !points.is_empty())
            .map(|(key, points)| TiledLeaf {
                key,
                geom: TiledGeom::Point(points),
            }),
    );

    (content, tiled)
}

// Extends `content`'s lng/lat bounds and projects to the normalized [0,1]
// Web Mercator space tile math uses.
fn project_ring(ring: &[Point], content: &mut TileContent) -> Vec<Point> {
    ring.iter()
        .map(|&[lng, lat]| {
            content.min_lng = content.min_lng.min(lng);
            content.max_lng = content.max_lng.max(lng);
            content.min_lat = content.min_lat.min(lat);
            content.max_lat = content.max_lat.max(lat);
            let (mx, my) = lnglat_to_web_mercator(lng, lat);
            [mx, my]
        })
        .collect()
}

fn tile_key(zoom: u8, xi: i64, yi: i64) -> TileKey {
    (
        zoom,
        xi.rem_euclid(1 << zoom) as u32,
        yi.rem_euclid(1 << zoom) as u32,
    )
}

fn ring_area(ring: &[Point]) -> f64 {
    let n = ring.len();
    if n < 3 {
        return 0.0;
    }
    let mut area = 0.0;
    for i in 0..n {
        let a = ring[i];
        let b = ring[(i + 1) % n];
        area += a[0] * b[1] - b[0] * a[1];
    }
    area / 2.0
}

// MVT requires CW exterior (opposite of Flow internals)
fn normalize_winding(mut rings: Vec<Vec<Point>>) -> (Vec<Point>, Vec<Vec<Point>>) {
    let holes = rings.split_off(1);
    let mut exterior = rings.remove(0);
    let exterior_area = ring_area(&exterior);
    if exterior_area > 0.0 {
        tracing::warn!(
            area = exterior_area,
            "MVT Writer: polygon exterior ring is not CCW"
        );
    } else {
        exterior.reverse();
    }
    let holes = holes
        .into_iter()
        .map(|mut hole| {
            let hole_area = ring_area(&hole);
            if hole_area < 0.0 {
                tracing::warn!(
                    area = hole_area,
                    "MVT Writer: polygon hole ring is not CW"
                );
            } else {
                hole.reverse();
            }
            hole
        })
        .collect();
    (exterior, holes)
}

fn interp(a: Point, b: Point, axis: usize, k: f64) -> Point {
    let other = 1 - axis;
    let t = (k - a[axis]) / (b[axis] - a[axis]);
    let mut p = [0.0; 2];
    p[axis] = k;
    p[other] = a[other] + t * (b[other] - a[other]);
    p
}

// Sutherland-Hodgman clip against the band [k1, k2] on axis (0 = x, 1 = y).
fn clip_band(points: &[Point], axis: usize, k1: f64, k2: f64, wraparound: bool) -> Vec<Point> {
    let n = points.len();
    if n == 0 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(n + 2);
    let edges = if wraparound { n } else { n - 1 };
    for i in 0..edges {
        let a = points[i];
        let b = points[(i + 1) % n];
        let (av, bv) = (a[axis], b[axis]);

        if av < k1 {
            if bv > k1 {
                out.push(interp(a, b, axis, k1));
            }
        } else if av > k2 {
            if bv < k2 {
                out.push(interp(a, b, axis, k2));
            }
        } else {
            out.push(a);
        }

        if bv < k1 && av > k1 {
            out.push(interp(a, b, axis, k1));
        } else if bv > k2 && av < k2 {
            out.push(interp(a, b, axis, k2));
        }
    }
    if !wraparound {
        let last = points[n - 1];
        if last[axis] >= k1 && last[axis] <= k2 {
            out.push(last);
        }
    }
    out
}

fn ring_bounds(ring: &[Point], axis: usize) -> (f64, f64) {
    ring.iter().fold((f64::MAX, f64::MIN), |(lo, hi), c| {
        (lo.min(c[axis]), hi.max(c[axis]))
    })
}

fn clip_polygon(
    exterior: &[Point],
    holes: &[Vec<Point>],
    y_bounds: (f64, f64),
    zoom: u8,
    out: &mut HashMap<TileKey, Vec<PolygonPart>>,
) {
    let z_scale = (1u64 << zoom) as f64;

    let (min_y, max_y) = y_bounds;
    let y_lo = (min_y * z_scale).floor() as i64;
    let y_hi = (max_y * z_scale).ceil() as i64;

    for yi in y_lo..y_hi {
        let k1 = (yi as f64 - MARGIN_WIDTH) / z_scale;
        let k2 = ((yi + 1) as f64 + MARGIN_WIDTH) / z_scale;
        let y_sliced_exterior = clip_band(exterior, 1, k1, k2, true);
        if y_sliced_exterior.is_empty() {
            continue;
        }
        let y_sliced_holes: Vec<Vec<Point>> = holes
            .iter()
            .map(|ring| clip_band(ring, 1, k1, k2, true))
            .collect();

        let (min_x, max_x) = ring_bounds(&y_sliced_exterior, 0);
        let x_lo = (min_x * z_scale).floor() as i64;
        let x_hi = (max_x * z_scale).ceil() as i64;

        for xi in x_lo..x_hi {
            let k1 = (xi as f64 - MARGIN_WIDTH) / z_scale;
            let k2 = ((xi + 1) as f64 + MARGIN_WIDTH) / z_scale;

            let to_local = |ring: &[Point]| -> Vec<Point> {
                let ring = clip_band(ring, 0, k1, k2, true);
                ring.iter()
                    .map(|&[x, y]| [x * z_scale - xi as f64, y * z_scale - yi as f64])
                    .collect()
            };

            let part_exterior = to_local(&y_sliced_exterior);
            if part_exterior.len() < 3 {
                continue;
            }
            let part_holes: Vec<Vec<Point>> = y_sliced_holes
                .iter()
                .map(|ring| to_local(ring))
                .filter(|h| h.len() >= 3)
                .collect();

            let key = tile_key(zoom, xi, yi);
            out.entry(key).or_default().push(PolygonPart {
                exterior: part_exterior,
                holes: part_holes,
            });
        }
    }
}

fn clip_line_string(
    line: &[Point],
    y_bounds: (f64, f64),
    zoom: u8,
    out: &mut HashMap<TileKey, Vec<Vec<Point>>>,
) {
    let z_scale = (1u64 << zoom) as f64;

    let (min_y, max_y) = y_bounds;
    let y_lo = (min_y * z_scale).floor() as i64;
    let y_hi = (max_y * z_scale).ceil() as i64;

    for yi in y_lo..y_hi {
        let k1 = (yi as f64 - MARGIN_WIDTH) / z_scale;
        let k2 = ((yi + 1) as f64 + MARGIN_WIDTH) / z_scale;
        let y_sliced = clip_band(line, 1, k1, k2, false);
        if y_sliced.is_empty() {
            continue;
        }

        let (min_x, max_x) = ring_bounds(&y_sliced, 0);
        let x_lo = (min_x * z_scale).floor() as i64;
        let x_hi = (max_x * z_scale).ceil() as i64;

        for xi in x_lo..x_hi {
            let k1 = (xi as f64 - MARGIN_WIDTH) / z_scale;
            let k2 = ((xi + 1) as f64 + MARGIN_WIDTH) / z_scale;

            let clipped = clip_band(&y_sliced, 0, k1, k2, false);
            if clipped.len() < 2 {
                continue;
            }
            let local: Vec<Point> = clipped
                .iter()
                .map(|&[x, y]| [x * z_scale - xi as f64, y * z_scale - yi as f64])
                .collect();

            let key = tile_key(zoom, xi, yi);
            out.entry(key).or_default().push(local);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_leaves_fixes_up_either_winding_to_mvts_required_cw() {
        // A lng/lat rectangle traced counter-clockwise (right, up, left, down).
        let ccw = vec![[0.0, 0.0], [90.0, 0.0], [90.0, 80.0], [0.0, 80.0]];
        let mut cw = ccw.clone();
        cw.reverse();

        for input in [ccw, cw] {
            let (_, tiled) = slice_leaves(vec![Leaf::Polygon(vec![input])], 0, 0, 16);
            let exteriors: Vec<_> = tiled
                .iter()
                .filter_map(|leaf| match &leaf.geom {
                    TiledGeom::Polygon(parts) => Some(parts),
                    _ => None,
                })
                .flatten()
                .collect();
            assert_eq!(exteriors.len(), 1);
            assert!(ring_area(&exteriors[0].exterior) > 0.0);
        }
    }
}
