use std::collections::HashMap;
use std::sync::Arc;

use reearth_flow_types::Attributes;
use tinymvt::geometry::GeometryEncoder;
use tinymvt::tag::TagsEncoder;
use tinymvt::vector_tile;

use super::slice::PolygonPart;
use crate::file::mvt::tags::convert_properties;

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
            [(x * extent as f64).round() as i32, (y * extent as f64).round() as i32]
        })
        .collect()
}

fn collinear(a: [i32; 2], b: [i32; 2], c: [i32; 2]) -> bool {
    let dx1 = (b[0] - a[0]) as i64;
    let dy1 = (b[1] - a[1]) as i64;
    let dx2 = (c[0] - a[0]) as i64;
    let dy2 = (c[1] - a[1]) as i64;
    dx1 * dy2 - dy1 * dx2 == 0
}

// One rule for every index, cyclic wraparound included for rings; an open
// chain's two endpoints never collapse since each has only one neighbor.
fn simplify(points: &[[i32; 2]], cyclic: bool) -> Vec<[i32; 2]> {
    let n = points.len();
    if n < 3 {
        return points.to_vec();
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        if !cyclic && (i == 0 || i == n - 1) {
            out.push(points[i]);
            continue;
        }
        let prev = points[(i + n - 1) % n];
        let curr = points[i];
        let next = points[(i + 1) % n];
        if !collinear(prev, curr, next) {
            out.push(curr);
        }
    }
    out
}

pub(super) fn make_tile(extent: i32, feats: &[SlicedFeature]) -> crate::errors::Result<Vec<u8>> {
    let mut layers: HashMap<String, LayerData> = HashMap::new();

    for feature in feats {
        let mut geom_enc = GeometryEncoder::new();
        let geom_type = match &feature.geom {
            SlicedGeom::Polygon(parts) => {
                for PolygonPart { exterior, holes } in parts {
                    let ring = simplify(&quantize(exterior, extent), true);
                    if ring.len() < 3 {
                        continue;
                    }
                    geom_enc.add_ring(ring);
                    for hole in holes {
                        let ring = simplify(&quantize(hole, extent), true);
                        if ring.len() >= 3 {
                            geom_enc.add_ring(ring);
                        }
                    }
                }
                vector_tile::tile::GeomType::Polygon
            }
            SlicedGeom::LineString(lines) => {
                for line in lines {
                    let line = simplify(&quantize(line, extent), false);
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
            continue;
        }

        let layer = layers.entry(feature.layer_name.clone()).or_default();
        for (key, value) in feature.properties.iter() {
            convert_properties(&mut layer.tags_enc, &key.inner().to_string(), value);
        }
        layer.features.push(vector_tile::tile::Feature {
            id: None,
            tags: layer.tags_enc.take_tags(),
            r#type: Some(geom_type as i32),
            geometry,
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

    Ok(prost::Message::encode_to_vec(&vector_tile::Tile { layers }))
}
