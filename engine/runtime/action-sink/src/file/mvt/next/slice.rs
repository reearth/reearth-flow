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
                bound(&mut content, rings.iter().flatten());

                let mercator = to_web_mercator(&rings);
                let (exterior, holes) = normalize_winding(mercator);
                let area = ring_area(&exterior).abs();

                for zoom in min_z..=max_z {
                    if area * (4u64.pow(zoom as u32 + max_detail) as f64) < 4.0 {
                        continue;
                    }
                    clip_polygon(&exterior, &holes, zoom, extent, buffer, &mut tiled_polys);
                }
            }
            Leaf::LineString(coords) => {
                bound(&mut content, coords.iter());

                let mercator: Ring = coords
                    .iter()
                    .map(|&[lng, lat]| {
                        let (mx, my) = lnglat_to_web_mercator(lng, lat);
                        [mx, my]
                    })
                    .collect();

                for zoom in min_z..=max_z {
                    clip_line_string(&mercator, zoom, extent, buffer, &mut tiled_lines);
                }
            }
            Leaf::Point([lng, lat]) => {
                bound(&mut content, std::iter::once(&[lng, lat]));

                let (mx, my) = lnglat_to_web_mercator(lng, lat);
                for zoom in min_z..=max_z {
                    let z_scale = (1u64 << zoom) as f64;
                    let xi = (mx * z_scale).floor() as i64;
                    let yi = (my * z_scale).floor() as i64;
                    let key: TileKey = (
                        zoom,
                        xi.rem_euclid(1 << zoom) as u32,
                        yi.rem_euclid(1 << zoom) as u32,
                    );
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

fn bound<'a>(content: &mut TileContent, coords: impl Iterator<Item = &'a [f64; 2]>) {
    for &[lng, lat] in coords {
        content.min_lng = content.min_lng.min(lng);
        content.max_lng = content.max_lng.max(lng);
        content.min_lat = content.min_lat.min(lat);
        content.max_lat = content.max_lat.max(lat);
    }
}

fn to_web_mercator(rings: &[Ring]) -> Vec<Ring> {
    rings
        .iter()
        .map(|ring| {
            ring.iter()
                .map(|&[lng, lat]| {
                    let (mx, my) = lnglat_to_web_mercator(lng, lat);
                    [mx, my]
                })
                .collect()
        })
        .collect()
}

// Shoelace formula, open/wraparound ring; positive = clockwise, matching tinymvt.
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

// Exterior negative, holes positive; the emit-time area filter in tile.rs expects this.
fn normalize_winding(mut rings: Vec<Ring>) -> (Ring, Vec<Ring>) {
    let mut holes = rings.split_off(1);
    let mut exterior = rings.remove(0);
    if ring_area(&exterior) > 0.0 {
        exterior.reverse();
    }
    for hole in &mut holes {
        if ring_area(hole) < 0.0 {
            hole.reverse();
        }
    }
    (exterior, holes)
}

fn interp(a: [f64; 2], b: [f64; 2], axis: usize, k: f64) -> [f64; 2] {
    let other = 1 - axis;
    let t = (k - a[axis]) / (b[axis] - a[axis]);
    let mut p = [0.0; 2];
    p[axis] = k;
    p[other] = a[other] + t * (b[other] - a[other]);
    p
}

// Clip an open, wraparound-closed ring to the band [k1, k2] on axis (0 = x, 1 = y).
fn clip_ring_band(ring: &Ring, axis: usize, k1: f64, k2: f64) -> Ring {
    let n = ring.len();
    if n == 0 {
        return Ring::new();
    }
    let mut out = Vec::with_capacity(n + 2);
    for i in 0..n {
        let a = ring[i];
        let b = ring[(i + 1) % n];
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
    out
}

fn ring_bounds(ring: &Ring, axis: usize) -> (f64, f64) {
    ring.iter()
        .fold((f64::MAX, f64::MIN), |(lo, hi), c| {
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
            .map(|ring| clip_ring_band(ring, 1, k1, k2))
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
                let ring = clip_ring_band(ring, 0, k1, k2);
                let mut local: Ring = ring
                    .iter()
                    .map(|&[x, y]| [x * z_scale - xi as f64, y * z_scale - yi as f64])
                    .collect();
                local.reverse();
                local
            });

            let part_exterior = clipped.next().expect("exterior always present");
            if part_exterior.len() < 3 {
                continue;
            }
            let part_holes: Vec<Ring> = clipped.filter(|h| h.len() >= 3).collect();

            let key: TileKey = (zoom, xi.rem_euclid(1 << zoom) as u32, yi.rem_euclid(1 << zoom) as u32);
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
        let y_sliced = clip_open_chain(line, 1, k1, k2);
        if y_sliced.is_empty() {
            continue;
        }

        let (min_x, max_x) = ring_bounds(&y_sliced, 0);
        let x_lo = (min_x * z_scale).floor() as i64;
        let x_hi = (max_x * z_scale).ceil() as i64;

        for xi in x_lo..x_hi {
            let k1 = (xi as f64 - buf_width) / z_scale;
            let k2 = ((xi + 1) as f64 + buf_width) / z_scale;

            let clipped = clip_open_chain(&y_sliced, 0, k1, k2);
            if clipped.len() < 2 {
                continue;
            }
            let local: Ring = clipped
                .iter()
                .map(|&[x, y]| [x * z_scale - xi as f64, y * z_scale - yi as f64])
                .collect();

            let key: TileKey = (zoom, xi.rem_euclid(1 << zoom) as u32, yi.rem_euclid(1 << zoom) as u32);
            out.entry(key).or_default().push(local);
        }
    }
}

// Like clip_ring_band, but an open (non-wraparound) chain.
fn clip_open_chain(chain: &Ring, axis: usize, k1: f64, k2: f64) -> Ring {
    let mut out = Vec::with_capacity(chain.len());
    for w in chain.windows(2) {
        let [a, b] = [w[0], w[1]];
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
    if let Some(&last) = chain.last() {
        if last[axis] >= k1 && last[axis] <= k2 {
            out.push(last);
        }
    }
    out
}
