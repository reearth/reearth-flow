use std::collections::HashMap;
use std::sync::Arc;

use reearth_flow_types::Attributes;
use tinymvt::geometry::GeometryEncoder;
use tinymvt::tag::TagsEncoder;
use tinymvt::vector_tile;

use super::slice::PolygonPart;
use crate::file::mvt::tags::convert_properties;

// Below this quantized bounding-box diagonal (extent-grid units) a feature covers less than a pixel; dropped unconditionally regardless of the size cap.
const SUBPIXEL_DIAMETER: f64 = 1.0;

pub(super) enum SlicedGeom {
    Polygon(Vec<PolygonPart>),
    LineString(Vec<Vec<[f64; 2]>>),
    Point(Vec<[f64; 2]>),
}

pub(super) struct SlicedFeature {
    pub(super) layer_name: String,
    pub(super) geom: SlicedGeom,
    pub(super) properties: Arc<Attributes>,
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

// Max perpendicular distance (in extent-grid units) a dropped point may sit
// from the simplified line; 1 unit keeps shape error within a single pixel at
// the tile's own resolution regardless of extent.
const SIMPLIFY_TOLERANCE: f64 = 1.0;

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

fn simplify(points: &[[i32; 2]], cyclic: bool) -> Vec<[i32; 2]> {
    let n = points.len();
    if n < 3 {
        return points.to_vec();
    }
    if !cyclic {
        return douglas_peucker(points, SIMPLIFY_TOLERANCE, 0);
    }

    let mut closed = points.to_vec();
    closed.push(points[0]);
    let mut simplified = douglas_peucker(&closed, SIMPLIFY_TOLERANCE, RING_RETAIN);
    simplified.pop();
    simplified
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

// One feature, already quantized/simplified/encoded and ready to write. `diameter` is `None` for
// points, which have no meaningful extent and are exempt from subpixel dropping.
struct Candidate {
    layer_name: String,
    properties: Arc<Attributes>,
    geom_type: vector_tile::tile::GeomType,
    geometry: Vec<u32>,
    diameter: Option<f64>,
}

fn build_candidate(extent: i32, feature: &SlicedFeature) -> Option<Candidate> {
    let mut geom_enc = GeometryEncoder::new();
    let mut min = [i32::MAX, i32::MAX];
    let mut max = [i32::MIN, i32::MIN];
    let geom_type = match &feature.geom {
        SlicedGeom::Polygon(parts) => {
            for PolygonPart { exterior, holes } in parts {
                let quantized = quantize(exterior, extent);
                extend_bounds(&quantized, &mut min, &mut max);
                geom_enc.add_ring(simplify(&quantized, true));
                for hole in holes {
                    let quantized = quantize(hole, extent);
                    extend_bounds(&quantized, &mut min, &mut max);
                    geom_enc.add_ring(simplify(&quantized, true));
                }
            }
            vector_tile::tile::GeomType::Polygon
        }
        SlicedGeom::LineString(lines) => {
            for line in lines {
                let quantized = quantize(line, extent);
                extend_bounds(&quantized, &mut min, &mut max);
                let line = simplify(&quantized, false);
                if line.len() >= 2 {
                    geom_enc.add_linestring(line);
                }
            }
            vector_tile::tile::GeomType::Linestring
        }
        SlicedGeom::Point(points) => {
            geom_enc.add_points(quantize(points, extent));
            vector_tile::tile::GeomType::Point
        }
    };

    let geometry = geom_enc.into_vec();
    if geometry.is_empty() {
        return None;
    }

    let diameter = (!matches!(feature.geom, SlicedGeom::Point(_))).then(|| {
        let (dx, dy) = ((max[0] - min[0]) as f64, (max[1] - min[1]) as f64);
        dx.hypot(dy)
    });

    Some(Candidate {
        layer_name: feature.layer_name.clone(),
        properties: feature.properties.clone(),
        geom_type,
        geometry,
        diameter,
    })
}

// Encodes `candidates` into a real tile and returns its exact bytes.
fn encode_tile(extent: i32, candidates: &[Candidate]) -> Vec<u8> {
    let mut layers: HashMap<String, LayerData> = HashMap::new();
    for candidate in candidates {
        let layer = layers.entry(candidate.layer_name.clone()).or_default();
        for (key, value) in candidate.properties.iter() {
            convert_properties(&mut layer.tags_enc, &key.inner().to_string(), value);
        }
        layer.features.push(vector_tile::tile::Feature {
            id: None,
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

pub(super) fn make_tile(
    extent: i32,
    feats: &[SlicedFeature],
    max_tile_bytes: u64,
) -> crate::errors::Result<Vec<u8>> {
    let mut candidates: Vec<Candidate> = feats
        .iter()
        .filter_map(|feature| build_candidate(extent, feature))
        .filter(|c| c.diameter.is_none_or(|d| d >= SUBPIXEL_DIAMETER))
        .collect();
    // Drop the visually smallest (by diameter) first when trimming for size.
    candidates.sort_by(|a, b| {
        a.diameter
            .unwrap_or(0.0)
            .total_cmp(&b.diameter.unwrap_or(0.0))
    });

    loop {
        let bytes = encode_tile(extent, &candidates);
        let actual = bytes.len() as u64;
        if actual <= max_tile_bytes || candidates.is_empty() {
            return Ok(bytes);
        }
        // Drop roughly (actual - max_tile_bytes) / actual of the smallest-diameter candidates
        // +1 floor guarantees halt; capped at len() so max_tile_bytes == 0 empties the tile
        // instead of overshooting split_off's valid range.
        let over = (actual - max_tile_bytes) as u128 * candidates.len() as u128;
        let drop = ((over / actual as u128) as usize + 1).min(candidates.len());
        candidates = candidates.split_off(drop);
    }
}

#[cfg(test)]
mod tests {
    use super::super::slice::Point;
    use super::*;

    #[test]
    fn drops_least_detailed_feature_to_fit_the_size_cap() {
        let extent = 4096;
        let n = 32;
        let big_ring: Vec<Point> = (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * i as f64 / n as f64;
                [0.5 + 0.4 * a.cos(), 0.5 + 0.4 * a.sin()]
            })
            .collect();
        let small_ring: Vec<Point> = vec![[0.1, 0.1], [0.2, 0.1], [0.15, 0.2]];

        let big_feature = SlicedFeature {
            layer_name: "layer".to_string(),
            geom: SlicedGeom::Polygon(vec![PolygonPart {
                exterior: big_ring.clone(),
                holes: vec![],
            }]),
            properties: Arc::new(Attributes::default()),
        };
        let small_feature = SlicedFeature {
            layer_name: "layer".to_string(),
            geom: SlicedGeom::Polygon(vec![PolygonPart {
                exterior: small_ring,
                holes: vec![],
            }]),
            properties: Arc::new(Attributes::default()),
        };
        let feats = vec![big_feature, small_feature];

        let both_bytes = make_tile(extent, &feats, u64::MAX).unwrap();
        let big_alone_bytes = make_tile(extent, &feats[..1], u64::MAX).unwrap();
        let cap = big_alone_bytes.len() as u64 + 1;
        assert!((cap as usize) < both_bytes.len());

        let bytes = make_tile(extent, &feats, cap).unwrap();
        assert!(bytes.len() as u64 <= cap);

        let tile: vector_tile::Tile = prost::Message::decode(bytes.as_slice()).unwrap();
        let feature_count: usize = tile.layers.iter().map(|l| l.features.len()).sum();
        assert_eq!(feature_count, 1);
        assert_eq!(
            tile.layers[0].features[0].geometry,
            build_candidate(extent, &feats[0]).unwrap().geometry
        );
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
        let quantized = simplify(&quantize(&exterior, extent), true);
        assert_eq!(quantized.len(), 3);
        assert!(shoelace_i32(&quantized) < 0.0);

        let feats = vec![SlicedFeature {
            layer_name: "layer".to_string(),
            geom: SlicedGeom::Polygon(vec![PolygonPart {
                exterior,
                holes: vec![],
            }]),
            properties: Arc::new(Attributes::default()),
        }];

        let bytes = make_tile(extent, &feats, u64::MAX).unwrap();
        let tile: vector_tile::Tile = prost::Message::decode(bytes.as_slice()).unwrap();
        let feature_count: usize = tile.layers.iter().map(|l| l.features.len()).sum();
        assert_eq!(feature_count, 1);
    }
}
