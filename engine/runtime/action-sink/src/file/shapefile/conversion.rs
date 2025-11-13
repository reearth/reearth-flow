use std::collections::HashMap;

use indexmap::IndexMap;
use reearth_flow_geometry::types::{
    multi_polygon::{MultiPolygon2D, MultiPolygon3D},
    polygon::{Polygon2D, Polygon3D},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, GeometryType, GeometryValue};
use shapefile::{
    dbase::{FieldName, FieldValue, Record, TableWriterBuilder},
    NO_DATA,
};

pub(super) fn feature_to_shape(feature: &Feature) -> crate::errors::Result<shapefile::Shape> {
    match &feature.geometry.value {
        GeometryValue::CityGmlGeometry(geometry) => {
            let mut mpoly = MultiPolygon3D::<f64>::default();
            geometry
                .gml_geometries
                .iter()
                .for_each(|entry| match entry.ty {
                    GeometryType::Solid | GeometryType::Surface | GeometryType::Triangle => {
                        entry.polygons.iter().for_each(|poly| {
                            mpoly.push(poly.clone());
                        });
                    }
                    GeometryType::Curve => unimplemented!(),
                    GeometryType::Point => unimplemented!(),
                });
            if !mpoly.is_empty() {
                let shape = shapefile::Shape::PolygonZ(multipolygons_to_shape(&mpoly));
                Ok(shape)
            } else {
                Ok(shapefile::Shape::NullShape)
            }
        }
        GeometryValue::FlowGeometry2D(geometry) => {
            use reearth_flow_geometry::types::geometry::Geometry2D;
            match geometry {
                Geometry2D::Point(point) => {
                    let shape =
                        shapefile::Shape::Point(shapefile::Point::new(point.x(), point.y()));
                    Ok(shape)
                }
                Geometry2D::MultiPoint(multi_point) => {
                    let points: Vec<shapefile::Point> = multi_point
                        .iter()
                        .map(|p| shapefile::Point::new(p.x(), p.y()))
                        .collect();
                    Ok(shapefile::Shape::Multipoint(shapefile::Multipoint::new(
                        points,
                    )))
                }
                Geometry2D::LineString(line_string) => {
                    let points: Vec<shapefile::Point> = line_string
                        .iter()
                        .map(|coord| shapefile::Point::new(coord.x, coord.y))
                        .collect();
                    Ok(shapefile::Shape::Polyline(shapefile::Polyline::new(points)))
                }
                Geometry2D::MultiLineString(multi_line_string) => {
                    let mut all_parts = Vec::new();
                    for line_string in multi_line_string.iter() {
                        let points: Vec<shapefile::Point> = line_string
                            .iter()
                            .map(|coord| shapefile::Point::new(coord.x, coord.y))
                            .collect();
                        all_parts.extend(points);
                    }
                    Ok(shapefile::Shape::Polyline(shapefile::Polyline::new(
                        all_parts,
                    )))
                }
                Geometry2D::Polygon(polygon) => {
                    let mpoly = MultiPolygon2D::new(vec![polygon.clone()]);
                    let shape = shapefile::Shape::Polygon(multipolygons_2d_to_shape(&mpoly));
                    Ok(shape)
                }
                Geometry2D::MultiPolygon(multi_polygon) => {
                    if !multi_polygon.is_empty() {
                        let shape =
                            shapefile::Shape::Polygon(multipolygons_2d_to_shape(multi_polygon));
                        Ok(shape)
                    } else {
                        Ok(shapefile::Shape::NullShape)
                    }
                }
                _ => Err(crate::errors::SinkError::ShapefileWriter(format!(
                    "Unsupported 2D geometry type: {}",
                    geometry.name()
                ))),
            }
        }
        GeometryValue::FlowGeometry3D(geometry) => {
            use reearth_flow_geometry::types::geometry::Geometry3D;
            match geometry {
                Geometry3D::Point(point) => {
                    let shape = shapefile::Shape::PointZ(shapefile::PointZ::new(
                        point.x(),
                        point.y(),
                        point.z(),
                        NO_DATA,
                    ));
                    Ok(shape)
                }
                Geometry3D::MultiPoint(multi_point) => {
                    let points: Vec<shapefile::PointZ> = multi_point
                        .iter()
                        .map(|p| shapefile::PointZ::new(p.x(), p.y(), p.z(), NO_DATA))
                        .collect();
                    Ok(shapefile::Shape::MultipointZ(shapefile::MultipointZ::new(
                        points,
                    )))
                }
                Geometry3D::LineString(line_string) => {
                    let points: Vec<shapefile::PointZ> = line_string
                        .iter()
                        .map(|coord| shapefile::PointZ::new(coord.x, coord.y, coord.z, NO_DATA))
                        .collect();
                    Ok(shapefile::Shape::PolylineZ(shapefile::PolylineZ::new(
                        points,
                    )))
                }
                Geometry3D::MultiLineString(multi_line_string) => {
                    let mut all_parts = Vec::new();
                    for line_string in multi_line_string.iter() {
                        let points: Vec<shapefile::PointZ> = line_string
                            .iter()
                            .map(|coord| shapefile::PointZ::new(coord.x, coord.y, coord.z, NO_DATA))
                            .collect();
                        all_parts.extend(points);
                    }
                    Ok(shapefile::Shape::PolylineZ(shapefile::PolylineZ::new(
                        all_parts,
                    )))
                }
                Geometry3D::Polygon(polygon) => {
                    let mpoly = MultiPolygon3D::new(vec![polygon.clone()]);
                    let shape = shapefile::Shape::PolygonZ(multipolygons_to_shape(&mpoly));
                    Ok(shape)
                }
                Geometry3D::MultiPolygon(multi_polygon) => {
                    if !multi_polygon.is_empty() {
                        let shape =
                            shapefile::Shape::PolygonZ(multipolygons_to_shape(multi_polygon));
                        Ok(shape)
                    } else {
                        Ok(shapefile::Shape::NullShape)
                    }
                }
                _ => Err(crate::errors::SinkError::ShapefileWriter(format!(
                    "Unsupported 3D geometry type: {}",
                    geometry.name()
                ))),
            }
        }
        GeometryValue::None => Ok(shapefile::Shape::NullShape),
    }
}

pub fn multipolygons_to_shape(mpoly: &MultiPolygon3D<f64>) -> shapefile::PolygonZ {
    let all_rings = mpoly
        .iter()
        .flat_map(polygon_to_shape_rings)
        .collect::<Vec<_>>();

    shapefile::PolygonZ::with_rings(all_rings)
}

pub fn multipolygons_2d_to_shape(mpoly: &MultiPolygon2D<f64>) -> shapefile::Polygon {
    let all_rings = mpoly
        .iter()
        .flat_map(polygon_2d_to_shape_rings)
        .collect::<Vec<_>>();

    shapefile::Polygon::with_rings(all_rings)
}

fn polygon_2d_to_shape_rings(
    poly: &Polygon2D<f64>,
) -> Vec<shapefile::PolygonRing<shapefile::Point>> {
    let outer_points = poly
        .exterior()
        .iter()
        .map(|coords| shapefile::Point::new(coords.x, coords.y))
        .collect::<Vec<shapefile::Point>>();
    let outer_ring = shapefile::PolygonRing::Outer(outer_points);

    let inner_rings = poly
        .interiors()
        .iter()
        .map(|ring| {
            ring.iter()
                .map(|coords| shapefile::Point::new(coords.x, coords.y))
                .collect::<Vec<shapefile::Point>>()
        })
        .map(shapefile::PolygonRing::Inner)
        .collect::<Vec<shapefile::PolygonRing<shapefile::Point>>>();

    let mut all_rings = vec![outer_ring];
    all_rings.extend(inner_rings);
    all_rings
}

fn polygon_to_shape_rings(poly: &Polygon3D<f64>) -> Vec<shapefile::PolygonRing<shapefile::PointZ>> {
    let outer_points = poly
        .exterior()
        .iter()
        .map(|coords| shapefile::PointZ::new(coords.x, coords.y, coords.z, NO_DATA))
        .collect::<Vec<shapefile::PointZ>>();
    let outer_ring = shapefile::PolygonRing::Outer(outer_points);

    let inner_rings = poly
        .interiors()
        .iter()
        .map(|ring| {
            ring.iter()
                .map(|coords| shapefile::PointZ::new(coords.x, coords.y, coords.z, NO_DATA))
                .collect::<Vec<shapefile::PointZ>>()
        })
        .map(shapefile::PolygonRing::Inner)
        .collect::<Vec<shapefile::PolygonRing<shapefile::PointZ>>>();

    let mut all_rings = vec![outer_ring];
    all_rings.extend(inner_rings);
    all_rings
}

pub(super) fn make_table_builder(
    attributes: &IndexMap<Attribute, AttributeValue>,
) -> crate::errors::Result<(TableWriterBuilder, HashMap<String, FieldValue>)> {
    let mut builder = TableWriterBuilder::new();
    let mut defaults = HashMap::new();

    for (field_name, attr) in attributes {
        let name: FieldName = trim_string_bytes(field_name.to_string(), 11)
            .as_str()
            .try_into()
            .map_err(|e| {
                crate::errors::SinkError::ShapefileWriter(format!(
                    "Failed to convert field name to FieldName: {e}"
                ))
            })?;
        let key = field_name.to_string();

        match attr {
            AttributeValue::String(_) => {
                builder = builder.add_character_field(name, 255);
                defaults.insert(key, FieldValue::Character(None));
            }
            AttributeValue::Number(num) => {
                if num.is_i64() {
                    builder = builder.add_numeric_field(name, 11, 0);
                    defaults.insert(key, FieldValue::Numeric(None));
                } else {
                    builder = builder.add_numeric_field(name, 18, 6);
                    defaults.insert(key, FieldValue::Numeric(None));
                }
            }
            AttributeValue::Bool(_) => {
                builder = builder.add_character_field(name, 6);
                defaults.insert(key, FieldValue::Character(None));
            }
            AttributeValue::DateTime(_) => {
                builder = builder.add_character_field(name, 255);
                defaults.insert(key, FieldValue::Character(None));
            }
            _ => {}
        }
    }
    Ok((builder, defaults))
}

pub(super) fn attributes_to_record(
    attributes: &IndexMap<Attribute, AttributeValue>,
    fields_default: &HashMap<String, FieldValue>,
) -> Record {
    let mut record = Record::default();

    // Fill in with default values for attributes that are not present
    for (name, default) in fields_default {
        if !attributes.contains_key(&Attribute::new(name)) {
            record.insert(name.to_string(), default.clone());
        }
    }

    for (attr_name, attr_value) in attributes {
        match attr_value {
            AttributeValue::String(s) => {
                // Shapefile cannot store string longer than 254 bytes
                let s = trim_string_bytes(s.clone(), 254);
                record.insert(attr_name.to_string(), FieldValue::Character(Some(s)));
            }
            AttributeValue::Number(num) => {
                record.insert(attr_name.to_string(), FieldValue::Numeric(num.as_f64()));
            }
            AttributeValue::Bool(b) => {
                record.insert(
                    attr_name.to_string(),
                    FieldValue::Character(Some(match b {
                        true => "true".to_string(),
                        false => "false".to_string(),
                    })),
                );
            }
            AttributeValue::DateTime(d) => {
                record.insert(
                    attr_name.to_string(),
                    FieldValue::Character(Some(d.to_rfc3339())),
                );
            }
            _ => {}
        };
    }
    record
}

fn trim_string_bytes(s: String, n: usize) -> String {
    let bytes = s.as_bytes();
    if bytes.len() <= n {
        return s;
    }
    match std::str::from_utf8(&bytes[..n]) {
        Ok(valid_str) => valid_str.to_string(),
        Err(e) => {
            let valid_up_to = e.valid_up_to();
            let valid_str = std::str::from_utf8(&bytes[..valid_up_to]).unwrap();
            valid_str.to_string()
        }
    }
}
