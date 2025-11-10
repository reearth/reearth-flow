//! Polygon slicing algorithm based on [geojson-vt](https://github.com/mapbox/geojson-vt).

use std::collections::{HashMap, HashSet};

use flatgeom::{MultiPolygon, Polygon, Polygon2, Polygon3};
use indexmap::IndexSet;
use itertools::Itertools;
use reearth_flow_types::{
    material::{self, Material},
    AttributeValue, Feature, GeometryType,
};
use serde::{Deserialize, Serialize};

use super::{tiling, tiling::zxy_from_lng_lat};

pub type TileZXYName = (u8, u32, u32);

#[derive(Serialize, Deserialize, Debug)]
pub struct SlicedFeature {
    // polygons [x, y, z, u, v]
    pub polygons: MultiPolygon<'static, [f64; 5]>,
    // material ids for each polygon
    pub polygon_material_ids: Vec<u32>,
    // materials
    pub materials: IndexSet<Material>,
    // attribute values
    pub attributes: HashMap<String, AttributeValue>,
}

pub fn slice_to_tiles<E>(
    feature: &Feature,
    schema: &nusamai_citygml::schema::Schema,
    min_zoom: u8,
    max_zoom: u8,
    attach_texture: bool,
    send_feature: impl Fn(TileZXYName, SlicedFeature) -> Result<(), E>,
) -> Result<(), E> {
    let Some(city_gml) = feature.geometry.value.as_citygml_geometry() else {
        return Ok(());
    };
    let ellipsoid = nusamai_projection::ellipsoid::wgs84();

    let slicing_enabled = false;

    let mut sliced_tiles: HashMap<(u8, u32, u32), SlicedFeature> = HashMap::new();
    let mut materials: IndexSet<Material> = IndexSet::new();
    let default_material = reearth_flow_types::material::X3DMaterial::default();

    let (lng_center, lat_center, approx_dx, approx_dy, approx_dh) = {
        let vertice = city_gml.max_min_vertice();
        let approx_dx = ellipsoid.a()
            * vertice.min_lat.to_radians().cos()
            * (vertice.max_lng - vertice.min_lng).to_radians();
        let approx_dy = ellipsoid.a() * (vertice.max_lng - vertice.min_lng).to_radians();
        let approx_dh = vertice.max_height - vertice.min_height;
        (
            (vertice.min_lng + vertice.max_lng) / 2.0,
            (vertice.min_lat + vertice.max_lat) / 2.0,
            approx_dx,
            approx_dy,
            approx_dh,
        )
    };
    let mut ring_buffer: Vec<[f64; 5]> = Vec::new();

    let available_lods: HashSet<u8> = city_gml
        .gml_geometries
        .iter()
        .flat_map(|entry| entry.lod)
        .sorted()
        .dedup()
        .collect();
    let Some(feature_type) = feature.feature_type() else {
        return Ok(());
    };
    let feature_schema = schema.types.get(&feature_type).unwrap();
    for entry in city_gml.gml_geometries.iter() {
        match entry.ty {
            GeometryType::Solid | GeometryType::Surface | GeometryType::Triangle => {
                // for each polygon
                for (((poly, poly_uv), poly_mat), poly_tex) in entry
                    .polygons
                    .iter()
                    .zip_eq(
                        city_gml
                            .polygon_uvs
                            .iter_range(entry.pos as usize..(entry.pos + entry.len) as usize),
                    )
                    .zip_eq(
                        city_gml.polygon_materials
                            [entry.pos as usize..(entry.pos + entry.len) as usize]
                            .iter(),
                    )
                    .zip_eq(
                        city_gml.polygon_textures
                            [entry.pos as usize..(entry.pos + entry.len) as usize]
                            .iter(),
                    )
                {
                    let poly: Polygon3 = poly.clone().into();
                    let mat = if attach_texture {
                        let orig_mat = poly_mat
                            .and_then(|idx| city_gml.materials.get(idx as usize))
                            .unwrap_or(&default_material)
                            .clone();
                        let orig_tex = poly_tex.and_then(|idx| city_gml.textures.get(idx as usize));
                        Material {
                            base_color: orig_mat.diffuse_color.into(),
                            base_texture: orig_tex
                                .map(|tex| material::Texture {
                                    uri: tex.uri.clone(),
                                })
                                .and_then(|tex| {
                                    if let Ok(file_path) = tex.uri.to_file_path() {
                                        if file_path.exists() {
                                            Some(tex)
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                }),
                        }
                    } else {
                        Material {
                            base_color: default_material.diffuse_color.into(),
                            base_texture: None,
                        }
                    };
                    let (mat_idx, _) = materials.insert_full(mat);
                    // Slice polygon for each zoom level
                    for zoom in min_zoom..=max_zoom {
                        // Don't filter at max_zoom - include all features at highest detail
                        if zoom < max_zoom {
                            let geom_error = {
                                let (_, _, y) =
                                    tiling::scheme::zxy_from_lng_lat(zoom, lng_center, lat_center);
                                tiling::scheme::geometric_error(zoom, y)
                            };
                            if let Some(lod) = entry.lod {
                                if !should_process_entry(lod, geom_error, &available_lods) {
                                    continue;
                                }
                            }

                            // Skip the feature if the size is small for geometricError.
                            let threshold = geom_error * 0.5;
                            if approx_dx < threshold
                                && approx_dy < threshold
                                && approx_dh < threshold
                            {
                                continue;
                            }
                        }

                        if slicing_enabled {
                            slice_polygon(zoom, &poly, &poly_uv, |(z, x, y), poly| {
                                let sliced_feature =
                                    sliced_tiles.entry((z, x, y)).or_insert_with(|| {
                                        SlicedFeature {
                                            polygons: MultiPolygon::new(),
                                            attributes: feature
                                                .attributes
                                                .clone()
                                                .into_iter()
                                                .filter(|(_, v)| v.convertible_nusamai_type_ref())
                                                .filter(|(k, _)| {
                                                    feature_schema.fields().contains(&k.to_string())
                                                })
                                                .map(|(k, v)| {
                                                    if let AttributeValue::Number(value) = v {
                                                        (
                                                            k.to_string(),
                                                            AttributeValue::String(
                                                                value.to_string(),
                                                            ),
                                                        )
                                                    } else {
                                                        (k.to_string(), v.clone())
                                                    }
                                                })
                                                .collect(),
                                            polygon_material_ids: Default::default(),
                                            materials: Default::default(), // set later
                                        }
                                    });
                                sliced_feature.polygons.push(poly);
                                sliced_feature.polygon_material_ids.push(mat_idx as u32);
                            });
                        } else {
                            let (z, x, y) = zxy_from_lng_lat(zoom, lng_center, lat_center);
                            let sliced_feature =
                                sliced_tiles
                                    .entry((z, x, y))
                                    .or_insert_with(|| SlicedFeature {
                                        polygons: MultiPolygon::new(),
                                        attributes: feature
                                            .attributes
                                            .clone()
                                            .into_iter()
                                            .filter(|(_, v)| v.convertible_nusamai_type_ref())
                                            .filter(|(k, _)| {
                                                feature_schema.fields().contains(&k.to_string())
                                            })
                                            .map(|(k, v)| match v {
                                                AttributeValue::Number(value) => (
                                                    k.to_string(),
                                                    AttributeValue::String(value.to_string()),
                                                ),
                                                AttributeValue::DateTime(value) => (
                                                    k.to_string(),
                                                    AttributeValue::String(value.to_string()),
                                                ),
                                                _ => (k.to_string(), v),
                                            })
                                            .collect(),
                                        polygon_material_ids: Default::default(),
                                        materials: Default::default(), // set later
                                    });
                            poly.rings().zip_eq(poly_uv.rings()).enumerate().for_each(
                                |(ri, (ring, uv_ring))| {
                                    ring.iter_closed().zip_eq(uv_ring.iter_closed()).for_each(
                                        |(c, uv)| {
                                            ring_buffer.push([c[0], c[1], c[2], uv[0], uv[1]]);
                                        },
                                    );
                                    if ri == 0 {
                                        sliced_feature.polygons.add_exterior(ring_buffer.drain(..));
                                        sliced_feature.polygon_material_ids.push(mat_idx as u32);
                                    } else {
                                        sliced_feature.polygons.add_interior(ring_buffer.drain(..));
                                    }
                                },
                            );
                        }
                    }
                }
            }
            GeometryType::Curve => {
                unimplemented!()
            }
            GeometryType::Point => {
                unimplemented!()
            }
        }
    }

    // Send tiled features
    for ((z, x, y), mut sliced_feature) in sliced_tiles {
        sliced_feature.materials.clone_from(&materials);
        send_feature((z, x, y), sliced_feature)?;
    }
    Ok(())

    // TODO: linestring, point
}

/// Slice a polygon into tiles. The slicing algorithm is based on [geojson-vt](https://github.com/mapbox/geojson-vt).
fn slice_polygon(
    zoom: u8,
    poly: &Polygon3,
    poly_uv: &Polygon2,
    mut send_polygon: impl FnMut(TileZXYName, &Polygon<'static, [f64; 5]>),
) {
    if poly.exterior().is_empty() {
        return;
    }

    let mut ring_buffer: Vec<[f64; 5]> = Vec::with_capacity(poly.exterior().len() + 1);

    // Slice along Y-axis
    let y_range = {
        let (min_y, max_y) = poly
            .exterior()
            .iter()
            .fold((f64::MAX, f64::MIN), |(min_y, max_y), c| {
                (min_y.min(c[1]), max_y.max(c[1]))
            });
        tiling::iter_y_slice(zoom, min_y, max_y)
    };

    let mut y_sliced_polys = MultiPolygon::new();

    for yi in y_range.clone() {
        let (k1, k2) = tiling::y_slice_range(zoom, yi);

        // todo?: check interior bbox to optimize

        for (ri, (ring, uv_ring)) in poly.rings().zip_eq(poly_uv.rings()).enumerate() {
            if ring.raw_coords().is_empty() {
                continue;
            }
            ring_buffer.clear();
            ring.iter_closed()
                .zip_eq(uv_ring.iter_closed())
                .fold(None, |a, b| {
                    let Some((a, a_uv)) = a else { return Some(b) };
                    let (b, b_uv) = b;

                    if a[1] < k1 {
                        if b[1] > k1 {
                            let t = (k1 - a[1]) / (b[1] - a[1]);
                            let x = (b[0] - a[0]) * t + a[0];
                            let z = (b[2] - a[2]) * t + a[2];
                            let u = (b_uv[0] - a_uv[0]) * t + a_uv[0];
                            let v = (b_uv[1] - a_uv[1]) * t + a_uv[1];
                            ring_buffer.push([x, k1, z, u, v])
                        }
                    } else if a[1] > k2 {
                        if b[1] < k2 {
                            let t = (k2 - a[1]) / (b[1] - a[1]);
                            let x = (b[0] - a[0]) * t + a[0];
                            let z = (b[2] - a[2]) * t + a[2];
                            let u = (b_uv[0] - a_uv[0]) * t + a_uv[0];
                            let v = (b_uv[1] - a_uv[1]) * t + a_uv[1];
                            ring_buffer.push([x, k2, z, u, v])
                        }
                    } else {
                        ring_buffer.push([a[0], a[1], a[2], a_uv[0], a_uv[1]])
                    }

                    if b[1] < k1 && a[1] > k1 {
                        let t = (k1 - a[1]) / (b[1] - a[1]);
                        let x = (b[0] - a[0]) * t + a[0];
                        let z = (b[2] - a[2]) * t + a[2];
                        let u = (b_uv[0] - a_uv[0]) * t + a_uv[0];
                        let v = (b_uv[1] - a_uv[1]) * t + a_uv[1];
                        ring_buffer.push([x, k1, z, u, v])
                    } else if b[1] > k2 && a[1] < k2 {
                        let t = (k2 - a[1]) / (b[1] - a[1]);
                        let x = (b[0] - a[0]) * t + a[0];
                        let z = (b[2] - a[2]) * t + a[2];
                        let u = (b_uv[0] - a_uv[0]) * t + a_uv[0];
                        let v = (b_uv[1] - a_uv[1]) * t + a_uv[1];
                        ring_buffer.push([x, k2, z, u, v])
                    }

                    Some((b, b_uv))
                })
                .unwrap();

            match ri {
                0 => y_sliced_polys.add_exterior(ring_buffer.drain(..)),
                _ => y_sliced_polys.add_interior(ring_buffer.drain(..)),
            }
        }
    }

    // Slice along X-axis
    let mut poly_buf: Polygon<[f64; 5]> = Polygon::new();
    for (yi, y_sliced_poly) in y_range.zip_eq(y_sliced_polys.iter()) {
        let x_iter = {
            let (min_x, max_x) = y_sliced_poly
                .exterior()
                .iter()
                .fold((f64::MAX, f64::MIN), |(min_x, max_x), c| {
                    (min_x.min(c[0]), max_x.max(c[0]))
                });
            tiling::iter_x_slice(zoom, yi, min_x, max_x)
        };

        for (xi, xs) in x_iter {
            let (k1, k2) = tiling::x_slice_range(zoom, xi, xs);

            // todo?: check interior bbox to optimize ...
            let x = xi.rem_euclid(1 << zoom) as u32;
            let key = (
                zoom, x, // handling geometry crossing the antimeridian
                yi,
            );
            poly_buf.clear();
            for ring in y_sliced_poly.rings() {
                if ring.raw_coords().is_empty() {
                    continue;
                }

                ring_buffer.clear();
                ring.iter_closed()
                    .fold(None, |a, b| {
                        let Some(a) = a else { return Some(b) };

                        if a[0] < k1 {
                            if b[0] > k1 {
                                let t = (k1 - a[0]) / (b[0] - a[0]);
                                let y = (b[1] - a[1]) * t + a[1];
                                let z = (b[2] - a[2]) * t + a[2];
                                let u = (b[3] - a[3]) * t + a[3];
                                let v = (b[4] - a[4]) * t + a[4];
                                ring_buffer.push([k1, y, z, u, v])
                            }
                        } else if a[0] > k2 {
                            if b[0] < k2 {
                                let t = (k2 - a[0]) / (b[0] - a[0]);
                                let y = (b[1] - a[1]) * t + a[1];
                                let z = (b[2] - a[2]) * t + a[2];
                                let u = (b[3] - a[3]) * t + a[3];
                                let v = (b[4] - a[4]) * t + a[4];
                                ring_buffer.push([k2, y, z, u, v])
                            }
                        } else {
                            ring_buffer.push(a)
                        }

                        if b[0] < k1 && a[0] > k1 {
                            let t = (k1 - a[0]) / (b[0] - a[0]);
                            let y = (b[1] - a[1]) * t + a[1];
                            let z = (b[2] - a[2]) * t + a[2];
                            let u = (b[3] - a[3]) * t + a[3];
                            let v = (b[4] - a[4]) * t + a[4];
                            ring_buffer.push([k1, y, z, u, v])
                        } else if b[0] > k2 && a[0] < k2 {
                            let t = (k2 - a[0]) / (b[0] - a[0]);
                            let y = (b[1] - a[1]) * t + a[1];
                            let z = (b[2] - a[2]) * t + a[2];
                            let u = (b[3] - a[3]) * t + a[3];
                            let v = (b[4] - a[4]) * t + a[4];
                            ring_buffer.push([k2, y, z, u, v])
                        }

                        Some(b)
                    })
                    .unwrap();

                poly_buf.add_ring(ring_buffer.drain(..))
            }
            send_polygon(key, &poly_buf);
        }
    }
}

fn desired_lod(geom_error: f64) -> u8 {
    match geom_error {
        ge if ge >= 30.0 => 1,
        ge if ge >= 20.0 => 2,
        ge if ge >= 5.0 => 3,
        _ => 4,
    }
}

fn should_process_entry(entry_lod: u8, geom_error: f64, available_lods: &HashSet<u8>) -> bool {
    let desired_lod = desired_lod(geom_error);

    let possible_lods: Vec<u8> = available_lods
        .iter()
        .cloned()
        .filter(|&lod| lod >= desired_lod)
        .collect();

    if !possible_lods.is_empty() {
        let selected_lod = *possible_lods.iter().min().unwrap();
        entry_lod == selected_lod
    } else {
        let selected_lod = *available_lods.iter().max().unwrap();
        entry_lod == selected_lod
    }
}
