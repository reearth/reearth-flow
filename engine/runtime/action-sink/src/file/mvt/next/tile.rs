use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use reearth_flow_types::Attributes;
use tinymvt::geometry::GeometryEncoder;
use tinymvt::tag::TagsEncoder;
use tinymvt::vector_tile;

use super::slice::PolygonPart;
use crate::file::mvt::tags::convert_scalar_property;

// Error budget of a full-detail tile: one unit of the nominal extent.
pub(super) const FULL_DETAIL: f64 = 1.0;

// Bisection of the error's exponent stops once the interval is narrower than
// this: the error is within a factor 2^BISECT_GAP of the largest that fits.
const BISECT_GAP: f64 = 0.125;

pub(super) enum SlicedGeom {
    Polygon(Vec<PolygonPart>),
    LineString(Vec<Vec<[f64; 2]>>),
    Point(Vec<[f64; 2]>),
}

pub(super) struct SlicedFeature {
    pub(super) layer_name: String,
    pub(super) geom: SlicedGeom,
    pub(super) properties: Arc<Attributes>,
    /// Vector-tile feature id, repeated in every tile the feature reaches.
    pub(super) id: Option<u64>,
    /// Index of the input feature this was sliced from.
    pub(super) source: usize,
}

#[derive(Default)]
struct LayerData {
    features: Vec<vector_tile::tile::Feature>,
    tags_enc: TagsEncoder,
}

fn quantize(points: &[[f64; 2]], extent: i32) -> Vec<[i32; 2]> {
    points
        .iter()
        .map(|&[x, y]| {
            [
                (x * extent as f64).round() as i32,
                (y * extent as f64).round() as i32,
            ]
        })
        .collect()
}

// Perpendicular distance from `points[i]` to the segment `points[start]..points[end]`,
// for every `i` in `start+1..end`; returns the farthest point and its distance.
fn farthest_from_segment(points: &[[i32; 2]], start: usize, end: usize) -> (usize, f64) {
    let [ax, ay] = points[start];
    let [bx, by] = points[end];
    let (dx, dy) = ((bx - ax) as f64, (by - ay) as f64);
    let len_sq = dx * dx + dy * dy;

    let mut best_idx = start + 1;
    let mut best_dist = -1.0;
    for (i, &[px, py]) in points.iter().enumerate().take(end).skip(start + 1) {
        let (ex, ey) = ((px - ax) as f64, (py - ay) as f64);
        let dist = if len_sq == 0.0 {
            (ex * ex + ey * ey).sqrt()
        } else {
            let t = ((ex * dx + ey * dy) / len_sq).clamp(0.0, 1.0);
            let (cx, cy) = (ax as f64 + t * dx, ay as f64 + t * dy);
            let (rx, ry) = (px as f64 - cx, py as f64 - cy);
            (rx * rx + ry * ry).sqrt()
        };
        if dist > best_dist {
            best_dist = dist;
            best_idx = i;
        }
    }
    (best_idx, best_dist)
}

// Polygon rings never simplify below this many points (as a closed ring, i.e.
// including the duplicated closing vertex; 3 distinct vertices), regardless of tolerance.
const RING_RETAIN: usize = 4;

// Ramer-Douglas-Peucker over an open chain; the two endpoints are always kept.
fn douglas_peucker(points: &[[i32; 2]], tolerance: f64, retain: usize) -> Vec<[i32; 2]> {
    let n = points.len();
    if n < 3 {
        return points.to_vec();
    }
    let mut keep = vec![false; n];
    keep[0] = true;
    keep[n - 1] = true;
    let mut kept = 2;
    let mut stack = vec![(0usize, n - 1)];
    while let Some((start, end)) = stack.pop() {
        if end <= start + 1 {
            continue;
        }
        let (idx, dist) = farthest_from_segment(points, start, end);
        if dist > tolerance || kept < retain {
            keep[idx] = true;
            kept += 1;
            stack.push((start, idx));
            stack.push((idx, end));
        }
    }
    points
        .iter()
        .zip(keep)
        .filter_map(|(&p, k)| k.then_some(p))
        .collect()
}

// `tolerance` is the max perpendicular distance, in extent units, a dropped
// point may sit from the simplified line.
fn simplify(points: &[[i32; 2]], cyclic: bool, tolerance: f64) -> Vec<[i32; 2]> {
    let n = points.len();
    if n < 3 {
        return points.to_vec();
    }
    if !cyclic {
        return douglas_peucker(points, tolerance, 0);
    }

    let mut closed = points.to_vec();
    closed.push(points[0]);
    let mut simplified = douglas_peucker(&closed, tolerance, RING_RETAIN);
    simplified.pop();
    simplified
}

/// Positions in `candidates` of the ones a tile keeps at `error`, ascending.
/// `candidates` are measured at full detail, so a line or polygon that
/// collapses even there draws nothing and is always left out. At full detail
/// everything else stays; beyond it, features at least `error` across stay,
/// and the smaller ones, points included, are thinned to the largest one
/// centred in each `error`-wide cell. Every cell that held a visible feature
/// still holds one, so no hole grows much past the cell, however dense the
/// features are.
///
/// Sizes are those of the pieces in this tile, clipped at its edges, so a
/// feature crossing an edge may be thinned in one tile and kept in the next.
fn select(candidates: &[Candidate], error: f64) -> Vec<usize> {
    let mut kept = Vec::new();
    let mut by_cell: HashMap<(i64, i64), (f64, usize)> = HashMap::new();
    for (position, candidate) in candidates.iter().enumerate() {
        if candidate.diameter == Some(0.0) {
            continue;
        }
        let size = candidate.diameter.unwrap_or(0.0);
        if error <= FULL_DETAIL || size >= error {
            kept.push(position);
            continue;
        }
        let cell = (
            (candidate.center[0] / error).floor() as i64,
            (candidate.center[1] / error).floor() as i64,
        );
        let best = by_cell.entry(cell).or_insert((size, position));
        if size > best.0 {
            *best = (size, position);
        }
    }
    kept.extend(by_cell.into_values().map(|(_, position)| position));
    kept.sort_unstable();
    kept
}

/// Every feature of a tile at full detail. Sizes and centres here do not
/// depend on the error budget, so they are measured once per tile.
fn measure(extent: i32, feats: &[SlicedFeature]) -> Vec<Candidate> {
    feats
        .iter()
        .enumerate()
        .filter_map(|(index, feature)| build_candidate(extent, FULL_DETAIL, index, feature))
        .collect()
}

/// The tile at `error`: the features [`select`] picks from `measured`, with
/// their lines and rings simplified within `error`.
fn tile_at(
    extent: i32,
    error: f64,
    feats: &[SlicedFeature],
    measured: &[Candidate],
) -> Vec<Candidate> {
    select(measured, error)
        .into_iter()
        .filter_map(|position| {
            let index = measured[position].index;
            build_candidate(extent, error, index, &feats[index])
        })
        .filter(|candidate| candidate.diameter != Some(0.0))
        .collect()
}

// One feature, already quantized/simplified/encoded and ready to write.
struct Candidate {
    /// Index into the tile's features.
    index: usize,
    source: usize,
    layer_name: String,
    id: Option<u64>,
    properties: Arc<Attributes>,
    geom_type: vector_tile::tile::GeomType,
    geometry: Vec<u32>,
    /// Bounding-box diagonal as encoded, in nominal extent units; `None` for
    /// points. Zero for a line or polygon that collapsed, which draws nothing.
    diameter: Option<f64>,
    /// Bounding-box centre as encoded, in nominal extent units.
    center: [f64; 2],
}

// Grows `[min, max]` to cover `points`.
fn extend_bounds(points: &[[i32; 2]], min: &mut [i32; 2], max: &mut [i32; 2]) {
    for &[x, y] in points {
        min[0] = min[0].min(x);
        min[1] = min[1].min(y);
        max[0] = max[0].max(x);
        max[1] = max[1].max(y);
    }
}

// Twice the signed area of an open ring.
fn doubled_area(ring: &[[i32; 2]]) -> i64 {
    (0..ring.len())
        .map(|i| {
            let ([ax, ay], [bx, by]) = (ring[i], ring[(i + 1) % ring.len()]);
            ax as i64 * by as i64 - bx as i64 * ay as i64
        })
        .sum()
}

fn build_candidate(
    extent: i32,
    error: f64,
    index: usize,
    feature: &SlicedFeature,
) -> Option<Candidate> {
    let mut geom_enc = GeometryEncoder::new();
    let mut min = [i32::MAX, i32::MAX];
    let mut max = [i32::MIN, i32::MIN];
    let mut has_area = false;
    let geom_type = match &feature.geom {
        SlicedGeom::Polygon(parts) => {
            for PolygonPart { exterior, holes } in parts {
                let exterior = simplify(&quantize(exterior, extent), true, error);
                extend_bounds(&exterior, &mut min, &mut max);
                has_area |= doubled_area(&exterior) != 0;
                geom_enc.add_ring(exterior);
                for hole in holes {
                    let hole = simplify(&quantize(hole, extent), true, error);
                    extend_bounds(&hole, &mut min, &mut max);
                    geom_enc.add_ring(hole);
                }
            }
            vector_tile::tile::GeomType::Polygon
        }
        SlicedGeom::LineString(lines) => {
            for line in lines {
                let line = simplify(&quantize(line, extent), false, error);
                if line.len() >= 2 {
                    extend_bounds(&line, &mut min, &mut max);
                    geom_enc.add_linestring(line);
                }
            }
            vector_tile::tile::GeomType::Linestring
        }
        SlicedGeom::Point(points) => {
            let points = quantize(points, extent);
            extend_bounds(&points, &mut min, &mut max);
            geom_enc.add_points(points);
            vector_tile::tile::GeomType::Point
        }
    };

    let geometry = geom_enc.into_vec();
    if geometry.is_empty() {
        return None;
    }

    let (dx, dy) = ((max[0] - min[0]) as f64, (max[1] - min[1]) as f64);
    let diameter = match &feature.geom {
        SlicedGeom::Point(_) => None,
        SlicedGeom::Polygon(_) if !has_area => Some(0.0),
        _ => Some(dx.hypot(dy)),
    };
    let center = [
        (min[0] as f64 + max[0] as f64) / 2.0,
        (min[1] as f64 + max[1] as f64) / 2.0,
    ];

    Some(Candidate {
        index,
        source: feature.source,
        layer_name: feature.layer_name.clone(),
        id: feature.id,
        properties: feature.properties.clone(),
        geom_type,
        geometry,
        diameter,
        center,
    })
}

// Encodes `candidates` into a real tile and returns its exact bytes.
fn encode_tile<'a>(extent: i32, candidates: impl Iterator<Item = &'a Candidate>) -> Vec<u8> {
    let mut layers: BTreeMap<String, LayerData> = BTreeMap::new();
    for candidate in candidates {
        let layer = layers.entry(candidate.layer_name.clone()).or_default();
        for (key, value) in candidate.properties.iter() {
            convert_scalar_property(&mut layer.tags_enc, &key.inner().to_string(), value);
        }
        layer.features.push(vector_tile::tile::Feature {
            id: candidate.id,
            tags: layer.tags_enc.take_tags(),
            r#type: Some(candidate.geom_type as i32),
            geometry: candidate.geometry.clone(),
        });
    }

    let layers = layers
        .into_iter()
        .flat_map(|(name, layer_data)| {
            if layer_data.features.is_empty() {
                return None;
            }
            let (keys, values) = layer_data.tags_enc.into_keys_and_values();
            Some(vector_tile::tile::Layer {
                version: 2,
                name,
                features: layer_data.features,
                keys,
                values,
                extent: Some(extent as u32),
            })
        })
        .collect();

    prost::Message::encode_to_vec(&vector_tile::Tile { layers })
}

/// An encoded tile, and which input features it holds.
pub(super) struct EncodedTile {
    pub(super) bytes: Vec<u8>,
    /// Sources of the features written into the tile. A source sliced into
    /// several parts here repeats.
    pub(super) written: Vec<usize>,
    /// Sources the size cap left out of the tile, repeating like `written`.
    /// Features that collapse at full detail are left out regardless, and are
    /// not here.
    pub(super) size_limited: Vec<usize>,
    /// The error budget the tile was written with, in nominal extent units;
    /// [`FULL_DETAIL`] for a tile at full detail.
    pub(super) error: f64,
}

/// Encode `feats` at full detail, or if that exceeds `max_tile_bytes`, at the
/// smallest error budget found by bisecting its exponent that fits. If it does
/// not fit once the budget spans the tile, the remaining features are trimmed
/// in input order.
pub(super) fn make_tile(
    extent: i32,
    feats: &[SlicedFeature],
    max_tile_bytes: u64,
) -> crate::errors::Result<EncodedTile> {
    let measured = measure(extent, feats);
    let at_full = select(&measured, FULL_DETAIL);
    let full: Vec<usize> = at_full.iter().map(|&p| measured[p].index).collect();
    let bytes = encode_tile(extent, at_full.iter().map(|&p| &measured[p]));
    if bytes.len() as u64 <= max_tile_bytes {
        return Ok(EncodedTile {
            bytes,
            written: at_full.iter().map(|&p| measured[p].source).collect(),
            size_limited: Vec::new(),
            error: FULL_DETAIL,
        });
    }

    // The error is 2^k. Full detail (k = 0) does not fit; the error spans the
    // tile at k = log2(2 * extent), as a clipped piece is under 2 extents
    // across. Bisect k between them, keeping the largest detail that fit.
    let attempt = |k: f64| {
        let error = k.exp2();
        let kept = tile_at(extent, error, feats, &measured);
        let bytes = encode_tile(extent, kept.iter());
        Attempt { error, kept, bytes }
    };
    let fits = |attempt: &Attempt| attempt.bytes.len() as u64 <= max_tile_bytes;
    let (mut lo, mut hi) = (0.0, (2.0 * extent as f64).log2());
    let mut best = attempt(hi);
    if fits(&best) {
        while hi - lo >= BISECT_GAP {
            let mid = (lo + hi) / 2.0;
            let candidate = attempt(mid);
            if fits(&candidate) {
                hi = mid;
                best = candidate;
            } else {
                lo = mid;
            }
        }
    }
    let Attempt {
        error,
        kept: mut current,
        mut bytes,
    } = best;
    while bytes.len() as u64 > max_tile_bytes && !current.is_empty() {
        // Drop roughly (actual - max_tile_bytes) / actual of the candidates; +1 floor
        // guarantees halt, capped at len() so max_tile_bytes == 0 empties the tile.
        let actual = bytes.len() as u64;
        let over = (actual - max_tile_bytes) as u128 * current.len() as u128;
        let drop = ((over / actual as u128) as usize + 1).min(current.len());
        current.drain(..drop);
        bytes = encode_tile(extent, current.iter());
    }

    let kept: HashSet<usize> = current.iter().map(|c| c.index).collect();
    Ok(EncodedTile {
        bytes,
        written: current.iter().map(|c| c.source).collect(),
        size_limited: full
            .into_iter()
            .filter(|index| !kept.contains(index))
            .map(|index| feats[index].source)
            .collect(),
        error,
    })
}

/// A tile encoded at one error budget.
struct Attempt {
    error: f64,
    kept: Vec<Candidate>,
    bytes: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::super::slice::Point;
    use super::*;

    fn polygon(source: usize, ring: Vec<Point>) -> SlicedFeature {
        SlicedFeature {
            layer_name: "layer".to_string(),
            geom: SlicedGeom::Polygon(vec![PolygonPart {
                exterior: ring,
                holes: vec![],
            }]),
            properties: Arc::new(Attributes::default()),
            id: None,
            source,
        }
    }

    /// A right triangle with legs of `size` tile widths.
    fn triangle(source: usize, size: f64) -> SlicedFeature {
        polygon(
            source,
            vec![[0.1, 0.1], [0.1 + size, 0.1], [0.1, 0.1 + size]],
        )
    }

    /// A big detailed ring and a small triangle.
    fn big_and_small() -> Vec<SlicedFeature> {
        let n = 256;
        let big_ring: Vec<Point> = (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * i as f64 / n as f64;
                [0.5 + 0.4 * a.cos(), 0.5 + 0.4 * a.sin()]
            })
            .collect();
        let small_ring: Vec<Point> = vec![[0.1, 0.1], [0.11, 0.1], [0.105, 0.11]];
        vec![polygon(0, big_ring), polygon(1, small_ring)]
    }

    /// A tile over the cap first gives up detail, with simpler rings, and keeps
    /// every feature.
    #[test]
    fn a_tile_over_the_cap_gives_up_detail_before_features() {
        let extent = 4096;
        let feats = big_and_small();
        let full = make_tile(extent, &feats, u64::MAX).unwrap();
        assert_eq!(full.error, FULL_DETAIL);

        let cap = full.bytes.len() as u64 - 1;
        let capped = make_tile(extent, &feats, cap).unwrap();
        assert!(capped.bytes.len() as u64 <= cap);
        assert!(capped.error > FULL_DETAIL);
        assert_eq!(capped.written, vec![0, 1]);
        assert!(capped.size_limited.is_empty());

        let tile: vector_tile::Tile = prost::Message::decode(capped.bytes.as_slice()).unwrap();
        assert_eq!(tile.layers[0].extent, Some(extent as u32));
    }

    /// Attributes that outweigh any geometry.
    fn heavy() -> Arc<Attributes> {
        let mut heavy = Attributes::default();
        heavy.insert(
            reearth_flow_types::Attribute::new("payload"),
            reearth_flow_types::AttributeValue::String("x".repeat(10_000)),
        );
        Arc::new(heavy)
    }

    /// When a small feature's attributes alone overflow the cap, the error
    /// grows until it just passes the big feature's diameter. The big one is
    /// then no longer kept outright, so it and the small one are thinned to the
    /// largest in their shared cell: the small one is dropped, the big one stays.
    #[test]
    fn a_growing_error_drops_the_smallest_features() {
        let extent = 4096;
        let mut feats = big_and_small();
        feats[1].properties = heavy();
        let big_diameter = measure(extent, &feats)[0].diameter.unwrap();

        let big_alone = make_tile(extent, &feats[..1], u64::MAX).unwrap();
        let capped = make_tile(extent, &feats, big_alone.bytes.len() as u64).unwrap();
        assert_eq!(capped.written, vec![0]);
        assert_eq!(capped.size_limited, vec![1]);
        assert!(capped.error > big_diameter);
        assert!(capped.error <= big_diameter * BISECT_GAP.exp2());
    }

    /// Two features too small to draw alone share a cell. Under a cap that
    /// holds one of them, one stays, not none.
    #[test]
    fn a_cap_that_holds_one_of_two_tiny_features_keeps_one() {
        let extent = 65536;
        let tiny = |source, x: f64| SlicedFeature {
            properties: heavy(),
            ..polygon(
                source,
                vec![[x, 0.5], [x + 0.0002, 0.5], [x + 0.0002, 0.5002]],
            )
        };
        let feats = vec![tiny(0, 0.5), tiny(1, 0.5003)];

        let one = make_tile(extent, &feats[..1], u64::MAX).unwrap();
        let capped = make_tile(extent, &feats, one.bytes.len() as u64).unwrap();
        assert_eq!(capped.written.len(), 1);
        assert_eq!(capped.size_limited.len(), 1);
    }

    #[test]
    fn layers_are_written_in_name_order() {
        let on_layer = |source, name: &str| SlicedFeature {
            layer_name: name.to_string(),
            ..polygon(source, vec![[0.1, 0.1], [0.2, 0.1], [0.1, 0.2]])
        };
        let feats = vec![on_layer(0, "b"), on_layer(1, "a")];

        let bytes = make_tile(4096, &feats, u64::MAX).unwrap().bytes;
        let tile: vector_tile::Tile = prost::Message::decode(bytes.as_slice()).unwrap();
        let names: Vec<&str> = tile.layers.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, ["a", "b"]);
    }

    #[test]
    fn a_zero_cap_keeps_nothing() {
        let tile = make_tile(4096, &big_and_small(), 0).unwrap();
        assert!(tile.written.is_empty());
        assert_eq!(tile.size_limited, vec![0, 1]);
    }

    /// A 32 by 32 block of squares 5 pixels wide, 8 pixels apart, covering the
    /// middle half of the tile.
    fn block() -> Vec<SlicedFeature> {
        (0..1024)
            .map(|i| {
                let (row, col) = ((i / 32) as f64, (i % 32) as f64);
                let (x, y) = (0.25 + col / 64.0, 0.25 + row / 64.0);
                let ring = vec![[x, y], [x + 0.01, y], [x + 0.01, y + 0.01], [x, y + 0.01]];
                polygon(i, ring)
            })
            .collect()
    }

    /// Thinning keeps one small feature per cell.
    #[test]
    fn thinning_keeps_one_feature_per_cell() {
        let extent = 4096;
        let feats = block();
        let measured = measure(extent, &feats);
        assert_eq!(select(&measured, FULL_DETAIL).len(), 1024);

        // Cells of 32 pixels hold 4 by 4 squares.
        let kept = tile_at(extent, extent as f64 / 16.0, &feats, &measured);
        assert_eq!(kept.len(), 64);
    }

    /// A feature that collapses even at full detail draws nothing whatever the
    /// cap, so it is not one the cap left out.
    #[test]
    fn a_feature_that_collapses_at_full_detail_is_not_size_limited() {
        let feats = vec![triangle(0, 0.5), triangle(1, 1e-6)];

        let tile = make_tile(4096, &feats, u64::MAX).unwrap();
        assert_eq!(tile.written, vec![0]);
        assert!(tile.size_limited.is_empty());
    }

    #[test]
    fn douglas_peucker_forced_retain_keeps_exactly_retain_distinct_points() {
        // Identical points force every split via `kept < retain`, so if an index could ever
        // be double-counted this would come up short of `retain`.
        let points = vec![[0, 0]; 10];
        let retain = 6;
        let result = douglas_peucker(&points, 0.0, retain);
        assert_eq!(result.len(), retain);
    }

    fn shoelace_f64(ring: &[[f64; 2]]) -> f64 {
        let n = ring.len();
        (0..n)
            .map(|i| {
                let [ax, ay] = ring[i];
                let [bx, by] = ring[(i + 1) % n];
                ax * by - bx * ay
            })
            .sum::<f64>()
            / 2.0
    }

    fn shoelace_i32(ring: &[[i32; 2]]) -> f64 {
        let n = ring.len();
        (0..n)
            .map(|i| {
                let [ax, ay] = ring[i];
                let [bx, by] = ring[(i + 1) % n];
                (ax * by - bx * ay) as f64
            })
            .sum::<f64>()
            / 2.0
    }

    #[test]
    fn a_ring_whose_area_sign_flips_under_quantization_is_still_encoded() {
        // Each vertex rounds to the extent grid independently, so a ring with positive
        // (pre-quantization) area can end up with negative area purely as a rounding artifact.
        let extent = 1;
        let exterior: Vec<Point> = vec![[0.0, 0.0], [100.0, 0.51], [50.0, 0.49]];
        assert!(shoelace_f64(&exterior) > 0.0);
        let quantized = simplify(&quantize(&exterior, extent), true, 1.0);
        assert_eq!(quantized.len(), 3);
        assert!(shoelace_i32(&quantized) < 0.0);

        let feats = vec![SlicedFeature {
            layer_name: "layer".to_string(),
            geom: SlicedGeom::Polygon(vec![PolygonPart {
                exterior,
                holes: vec![],
            }]),
            properties: Arc::new(Attributes::default()),
            id: None,
            source: 0,
        }];

        let bytes = make_tile(extent, &feats, u64::MAX).unwrap().bytes;
        let tile: vector_tile::Tile = prost::Message::decode(bytes.as_slice()).unwrap();
        let feature_count: usize = tile.layers.iter().map(|l| l.features.len()).sum();
        assert_eq!(feature_count, 1);
    }
}
