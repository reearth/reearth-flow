use std::collections::HashMap;

use tinymvt::webmercator::lnglat_to_web_mercator;

use super::extract::Leaf;
use crate::file::mvt::tiling::TileContent;

pub(super) type Ring = Vec<[f64; 2]>;
pub(super) type TileKey = (u8, u32, u32);

pub(super) struct PolygonPart {
    pub(super) exterior: Ring,
    pub(super) holes: Vec<Ring>,
}

pub(super) enum TiledGeom {
    Polygon(Vec<PolygonPart>),
    /// Separate line paths landing in the same tile.
    LineString(Vec<Ring>),
    Point(Vec<[f64; 2]>),
}

pub(super) struct TiledLeaf {
    pub(super) key: TileKey,
    pub(super) geom: TiledGeom,
}

pub(super) fn slice_leaves(
    leaves: Vec<Leaf>,
    min_z: u8,
    max_z: u8,
    max_detail: u32,
    buffer_pixels: u32,
) -> (TileContent, Vec<TiledLeaf>) {
    let mut tiled_polys: HashMap<TileKey, Vec<PolygonPart>> = HashMap::new();
    let mut tiled_lines: HashMap<TileKey, Vec<Ring>> = HashMap::new();
    let mut tiled_points: HashMap<TileKey, Vec<[f64; 2]>> = HashMap::new();
    let mut content = TileContent::default();

    let extent = 1 << max_detail;
    let buffer = extent * buffer_pixels / 256;

    for leaf in leaves {
        match leaf {
            Leaf::Polygon(rings) => {
                let mercator: Vec<Ring> = rings
                    .iter()
                    .map(|ring| project_ring(ring, &mut content))
                    .collect();
                let Some((exterior, holes)) = normalize_winding(mercator) else {
                    continue;
                };
                let area = ring_area(&exterior).abs();

                for zoom in min_z..=max_z {
                    if area * 4f64.powi(zoom as i32 + max_detail as i32) < 4.0 {
                        continue;
                    }
                    clip_polygon(&exterior, &holes, zoom, extent, buffer, &mut tiled_polys);
                }
            }
            Leaf::LineString(coords) => {
                let mercator = project_ring(&coords, &mut content);

                for zoom in min_z..=max_z {
                    clip_line_string(&mercator, zoom, extent, buffer, &mut tiled_lines);
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
fn project_ring(ring: &[[f64; 2]], content: &mut TileContent) -> Ring {
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

fn ring_area(ring: &[[f64; 2]]) -> f64 {
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

fn normalize_winding(mut rings: Vec<Ring>) -> Option<(Ring, Vec<Ring>)> {
    let holes = rings.split_off(1);
    let exterior = rings.remove(0);
    if ring_area(&exterior) > 0.0 {
        tracing::error!("MVT Writer: polygon exterior ring is not CCW as required; skipping polygon");
        return None;
    }
    let holes = holes
        .into_iter()
        .filter(|hole| {
            if ring_area(hole) < 0.0 {
                tracing::error!("MVT Writer: polygon hole ring is not CW as required; skipping hole");
                false
            } else {
                true
            }
        })
        .collect();
    Some((exterior, holes))
}

fn interp(a: [f64; 2], b: [f64; 2], axis: usize, k: f64) -> [f64; 2] {
    let other = 1 - axis;
    let t = (k - a[axis]) / (b[axis] - a[axis]);
    let mut p = [0.0; 2];
    p[axis] = k;
    p[other] = a[other] + t * (b[other] - a[other]);
    p
}

// Sutherland-Hodgman clip against the band [k1, k2] on axis (0 = x, 1 = y).
fn clip_band(points: &Ring, axis: usize, k1: f64, k2: f64, wraparound: bool) -> Ring {
    let n = points.len();
    if n == 0 {
        return Ring::new();
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

fn ring_bounds(ring: &Ring, axis: usize) -> (f64, f64) {
    ring.iter().fold((f64::MAX, f64::MIN), |(lo, hi), c| {
        (lo.min(c[axis]), hi.max(c[axis]))
    })
}

fn clip_polygon(
    exterior: &Ring,
    holes: &[Ring],
    zoom: u8,
    extent: u32,
    buffer: u32,
    out: &mut HashMap<TileKey, Vec<PolygonPart>>,
) {
    let z_scale = (1u64 << zoom) as f64;
    let buf_width = buffer as f64 / extent as f64;

    let rings: Vec<&Ring> = std::iter::once(exterior).chain(holes.iter()).collect();

    let (min_y, max_y) = ring_bounds(exterior, 1);
    let y_lo = (min_y * z_scale).floor() as i64;
    let y_hi = (max_y * z_scale).ceil() as i64;

    for yi in y_lo..y_hi {
        let k1 = (yi as f64 - buf_width) / z_scale;
        let k2 = ((yi + 1) as f64 + buf_width) / z_scale;
        let y_sliced: Vec<Ring> = rings
            .iter()
            .map(|ring| clip_band(ring, 1, k1, k2, true))
            .collect();
        if y_sliced[0].is_empty() {
            continue;
        }

        let (min_x, max_x) = ring_bounds(&y_sliced[0], 0);
        let x_lo = (min_x * z_scale).floor() as i64;
        let x_hi = (max_x * z_scale).ceil() as i64;

        for xi in x_lo..x_hi {
            let k1 = (xi as f64 - buf_width) / z_scale;
            let k2 = ((xi + 1) as f64 + buf_width) / z_scale;

            let mut clipped = y_sliced.iter().map(|ring| {
                let ring = clip_band(ring, 0, k1, k2, true);
                let mut local: Ring = ring
                    .iter()
                    .map(|&[x, y]| [x * z_scale - xi as f64, y * z_scale - yi as f64])
                    .collect();
                // MVT requires clockwise winding.
                local.reverse();
                local
            });

            let part_exterior = clipped.next().expect("exterior always present");
            if part_exterior.len() < 3 {
                continue;
            }
            let part_holes: Vec<Ring> = clipped.filter(|h| h.len() >= 3).collect();

            let key = tile_key(zoom, xi, yi);
            out.entry(key).or_default().push(PolygonPart {
                exterior: part_exterior,
                holes: part_holes,
            });
        }
    }
}

fn clip_line_string(
    line: &Ring,
    zoom: u8,
    extent: u32,
    buffer: u32,
    out: &mut HashMap<TileKey, Vec<Ring>>,
) {
    let z_scale = (1u64 << zoom) as f64;
    let buf_width = buffer as f64 / extent as f64;

    let (min_y, max_y) = ring_bounds(line, 1);
    let y_lo = (min_y * z_scale).floor() as i64;
    let y_hi = (max_y * z_scale).ceil() as i64;

    for yi in y_lo..y_hi {
        let k1 = (yi as f64 - buf_width) / z_scale;
        let k2 = ((yi + 1) as f64 + buf_width) / z_scale;
        let y_sliced = clip_band(line, 1, k1, k2, false);
        if y_sliced.is_empty() {
            continue;
        }

        let (min_x, max_x) = ring_bounds(&y_sliced, 0);
        let x_lo = (min_x * z_scale).floor() as i64;
        let x_hi = (max_x * z_scale).ceil() as i64;

        for xi in x_lo..x_hi {
            let k1 = (xi as f64 - buf_width) / z_scale;
            let k2 = ((xi + 1) as f64 + buf_width) / z_scale;

            let clipped = clip_band(&y_sliced, 0, k1, k2, false);
            if clipped.len() < 2 {
                continue;
            }
            let local: Ring = clipped
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
    fn slice_leaves_drops_only_the_wrongly_wound_polygon() {
        // A lng/lat rectangle traced counter-clockwise (right, up, left, down); its Web Mercator
        // projection is verified by hand to have negative `ring_area`, the winding
        // `normalize_winding` requires of an exterior ring.
        let ccw = vec![[0.0, 0.0], [90.0, 0.0], [90.0, 80.0], [0.0, 80.0]];
        let mut cw = ccw.clone();
        cw.reverse();

        let (_, tiled) = slice_leaves(vec![Leaf::Polygon(vec![ccw])], 0, 0, 4, 0);
        let polygon_count = tiled
            .iter()
            .filter(|leaf| matches!(leaf.geom, TiledGeom::Polygon(_)))
            .count();
        assert_eq!(polygon_count, 1);

        let (_, tiled) = slice_leaves(vec![Leaf::Polygon(vec![cw])], 0, 0, 4, 0);
        let polygon_count = tiled
            .iter()
            .filter(|leaf| matches!(leaf.geom, TiledGeom::Polygon(_)))
            .count();
        assert_eq!(polygon_count, 0);
    }
}
