use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_geometry::types::{
    coordinate::Coordinate3D, line_string::LineString3D, polygon::Polygon3D,
};
use reearth_flow_runtime::node::{IngestionMessage, Port, DEFAULT_PORT};
use reearth_flow_types::{
    Attribute, AttributeValue, CityGmlGeometry, Feature, Geometry, GeometryType, GeometryValue,
    GmlGeometry,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read};
use tokio::sync::mpsc::Sender;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShapefileReaderParam {
    pub encoding: Option<String>,
}

pub(crate) async fn read_shapefile(
    content: &Bytes,
    _params: &ShapefileReaderParam,
    sender: Sender<(Port, IngestionMessage)>,
) -> Result<(), crate::errors::SourceError> {
    let shapes_and_records = if is_zip_file(content) {
        read_shapefile_from_zip(content)?
    } else {
        return Err(crate::errors::SourceError::ShapefileReader(
            "Direct shapefile bytes not supported. Please provide a ZIP archive containing the shapefile components (.shp, .dbf, .shx)".to_string()
        ));
    };

    for (shape, record) in shapes_and_records {
        let geometry = convert_shape_to_geometry(shape)?;
        let attributes = convert_record_to_attributes(record);

        let feature = Feature {
            geometry,
            attributes,
            ..Default::default()
        };

        sender
            .send((
                DEFAULT_PORT.clone(),
                IngestionMessage::OperationEvent { feature },
            ))
            .await
            .map_err(|e| {
                crate::errors::SourceError::ShapefileReader(format!("Failed to send feature: {e}"))
            })?;
    }

    Ok(())
}

fn is_zip_file(content: &Bytes) -> bool {
    content.len() >= 2 && content[0] == 0x50 && content[1] == 0x4B
}

fn read_shapefile_from_zip(
    content: &Bytes,
) -> Result<Vec<(shapefile::Shape, shapefile::dbase::Record)>, crate::errors::SourceError> {
    let cursor = Cursor::new(content.as_ref());
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| {
        crate::errors::SourceError::ShapefileReader(format!("Failed to read ZIP archive: {e}"))
    })?;

    let mut shp_data: Option<Vec<u8>> = None;
    let mut dbf_data: Option<Vec<u8>> = None;
    let mut _shx_data: Option<Vec<u8>> = None;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            crate::errors::SourceError::ShapefileReader(format!("Failed to read ZIP entry: {e}"))
        })?;

        let file_name = file.name().to_lowercase();

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| {
            crate::errors::SourceError::ShapefileReader(format!("Failed to read ZIP entry: {e}"))
        })?;

        if file_name.ends_with(".shp") {
            shp_data = Some(buffer);
        } else if file_name.ends_with(".dbf") {
            dbf_data = Some(buffer);
        } else if file_name.ends_with(".shx") {
            _shx_data = Some(buffer);
        }
    }

    let shp_data = shp_data.ok_or_else(|| {
        crate::errors::SourceError::ShapefileReader("No .shp file found in ZIP archive".to_string())
    })?;
    let dbf_data = dbf_data.ok_or_else(|| {
        crate::errors::SourceError::ShapefileReader("No .dbf file found in ZIP archive".to_string())
    })?;

    let shp_cursor = Cursor::new(shp_data);
    let dbf_cursor = Cursor::new(dbf_data);

    let shape_reader = shapefile::ShapeReader::new(shp_cursor).map_err(|e| {
        crate::errors::SourceError::ShapefileReader(format!("Failed to create shape reader: {e}"))
    })?;

    let dbf_reader = shapefile::dbase::Reader::new(dbf_cursor).map_err(|e| {
        crate::errors::SourceError::ShapefileReader(format!("Failed to create DBF reader: {e}"))
    })?;

    let mut reader = shapefile::Reader::new(shape_reader, dbf_reader);

    let mut shapes_and_records = Vec::new();
    for result in reader.iter_shapes_and_records() {
        let (shape, record) = result.map_err(|e| {
            crate::errors::SourceError::ShapefileReader(format!("Failed to read shape/record: {e}"))
        })?;
        shapes_and_records.push((shape, record));
    }

    Ok(shapes_and_records)
}

fn create_point_geometry(coord: Coordinate3D<f64>) -> GeometryValue {
    let line_string = LineString3D::from(vec![coord]);
    let entry = GmlGeometry {
        ty: GeometryType::Point,
        polygons: vec![],
        line_strings: vec![line_string],
        id: None,
        lod: None,
        pos: 0,
        len: 0,
        feature_id: None,
        feature_type: None,
        composite_surfaces: vec![],
    };
    GeometryValue::CityGmlGeometry(CityGmlGeometry {
        gml_geometries: vec![entry],
        ..Default::default()
    })
}

fn convert_shape_to_geometry(
    shape: shapefile::Shape,
) -> Result<Geometry, crate::errors::SourceError> {
    let geometry_value = match shape {
        shapefile::Shape::NullShape => GeometryValue::None,
        shapefile::Shape::Point(point) => create_point_geometry(Coordinate3D {
            x: point.x,
            y: point.y,
            z: 0.0,
        }),
        shapefile::Shape::PointM(point) => create_point_geometry(Coordinate3D {
            x: point.x,
            y: point.y,
            z: 0.0,
        }),
        shapefile::Shape::PointZ(point) => create_point_geometry(Coordinate3D {
            x: point.x,
            y: point.y,
            z: point.z,
        }),
        shapefile::Shape::Polyline(polyline) => {
            create_polyline_geometry(polyline.parts().iter().map(|part| {
                part.iter()
                    .map(|p| Coordinate3D {
                        x: p.x,
                        y: p.y,
                        z: 0.0,
                    })
                    .collect()
            }))
        }
        shapefile::Shape::PolylineM(polyline) => {
            create_polyline_geometry(polyline.parts().iter().map(|part| {
                part.iter()
                    .map(|p| Coordinate3D {
                        x: p.x,
                        y: p.y,
                        z: 0.0,
                    })
                    .collect()
            }))
        }
        shapefile::Shape::PolylineZ(polyline) => {
            create_polyline_geometry(polyline.parts().iter().map(|part| {
                part.iter()
                    .map(|p| Coordinate3D {
                        x: p.x,
                        y: p.y,
                        z: p.z,
                    })
                    .collect()
            }))
        }
        shapefile::Shape::Polygon(polygon) => {
            create_polygon_geometry(polygon.rings().iter().map(|ring| {
                match ring {
                    shapefile::PolygonRing::Outer(points) => (
                        true,
                        points
                            .iter()
                            .map(|p| Coordinate3D {
                                x: p.x,
                                y: p.y,
                                z: 0.0,
                            })
                            .collect(),
                    ),
                    shapefile::PolygonRing::Inner(points) => (
                        false,
                        points
                            .iter()
                            .map(|p| Coordinate3D {
                                x: p.x,
                                y: p.y,
                                z: 0.0,
                            })
                            .collect(),
                    ),
                }
            }))?
        }
        shapefile::Shape::PolygonM(polygon) => {
            create_polygon_geometry(polygon.rings().iter().map(|ring| {
                match ring {
                    shapefile::PolygonRing::Outer(points) => (
                        true,
                        points
                            .iter()
                            .map(|p| Coordinate3D {
                                x: p.x,
                                y: p.y,
                                z: 0.0,
                            })
                            .collect(),
                    ),
                    shapefile::PolygonRing::Inner(points) => (
                        false,
                        points
                            .iter()
                            .map(|p| Coordinate3D {
                                x: p.x,
                                y: p.y,
                                z: 0.0,
                            })
                            .collect(),
                    ),
                }
            }))?
        }
        shapefile::Shape::PolygonZ(polygon) => {
            create_polygon_geometry(polygon.rings().iter().map(|ring| {
                match ring {
                    shapefile::PolygonRing::Outer(points) => (
                        true,
                        points
                            .iter()
                            .map(|p| Coordinate3D {
                                x: p.x,
                                y: p.y,
                                z: p.z,
                            })
                            .collect(),
                    ),
                    shapefile::PolygonRing::Inner(points) => (
                        false,
                        points
                            .iter()
                            .map(|p| Coordinate3D {
                                x: p.x,
                                y: p.y,
                                z: p.z,
                            })
                            .collect(),
                    ),
                }
            }))?
        }
        shapefile::Shape::Multipoint(_)
        | shapefile::Shape::MultipointM(_)
        | shapefile::Shape::MultipointZ(_) => {
            return Err(crate::errors::SourceError::ShapefileReader(
                "Multipoint shapes not yet supported".to_string(),
            ));
        }
        shapefile::Shape::Multipatch(_) => {
            return Err(crate::errors::SourceError::ShapefileReader(
                "Multipatch shapes not yet supported".to_string(),
            ));
        }
    };

    Ok(Geometry {
        value: geometry_value,
        ..Default::default()
    })
}

fn create_polyline_geometry(parts: impl Iterator<Item = Vec<Coordinate3D<f64>>>) -> GeometryValue {
    let line_strings: Vec<_> = parts.map(LineString3D::from).collect();
    let entry = GmlGeometry {
        ty: GeometryType::Curve,
        polygons: vec![],
        line_strings,
        id: None,
        lod: None,
        pos: 0,
        len: 0,
        feature_id: None,
        feature_type: None,
        composite_surfaces: vec![],
    };
    GeometryValue::CityGmlGeometry(CityGmlGeometry {
        gml_geometries: vec![entry],
        ..Default::default()
    })
}

fn create_polygon_geometry(
    rings: impl Iterator<Item = (bool, Vec<Coordinate3D<f64>>)>,
) -> Result<GeometryValue, crate::errors::SourceError> {
    let mut polygons = Vec::new();

    for (is_outer, coords) in rings {
        if is_outer {
            let poly = Polygon3D::new(LineString3D::from(coords), vec![]);
            polygons.push(poly);
        } else {
            // Note: The shapefile spec guarantees that inner rings follow their outer ring
            // If no outer ring exists, we skip the inner ring with a warning
            if let Some(last_poly) = polygons.last_mut() {
                last_poly.interiors_push(LineString3D::from(coords));
            }
            // If there's no polygon to attach to, the shapefile may be malformed
        }
    }

    let entry = GmlGeometry {
        ty: GeometryType::Surface,
        polygons,
        line_strings: vec![],
        id: None,
        lod: None,
        pos: 0,
        len: 0,
        feature_id: None,
        feature_type: None,
        composite_surfaces: vec![],
    };

    Ok(GeometryValue::CityGmlGeometry(CityGmlGeometry {
        gml_geometries: vec![entry],
        ..Default::default()
    }))
}

fn convert_record_to_attributes(
    record: shapefile::dbase::Record,
) -> IndexMap<Attribute, AttributeValue> {
    let mut attributes = IndexMap::new();

    for (name, value) in record {
        let attr_name = Attribute::new(name);
        let attr_value = match value {
            shapefile::dbase::FieldValue::Character(Some(s)) => AttributeValue::String(s),
            shapefile::dbase::FieldValue::Numeric(Some(n)) => {
                // Handle NaN, Infinity, and other invalid JSON numbers
                if n.is_finite() {
                    serde_json::Number::from_f64(n)
                        .map(AttributeValue::Number)
                        .unwrap_or(AttributeValue::Null)
                } else {
                    AttributeValue::Null
                }
            }
            shapefile::dbase::FieldValue::Logical(Some(b)) => AttributeValue::Bool(b),
            shapefile::dbase::FieldValue::Date(Some(d)) => {
                AttributeValue::String(format!("{:04}-{:02}-{:02}", d.year(), d.month(), d.day()))
            }
            shapefile::dbase::FieldValue::Float(Some(f)) => {
                let f64_val = f64::from(f);
                // Handle NaN, Infinity, and other invalid JSON numbers
                if f64_val.is_finite() {
                    serde_json::Number::from_f64(f64_val)
                        .map(AttributeValue::Number)
                        .unwrap_or(AttributeValue::Null)
                } else {
                    AttributeValue::Null
                }
            }
            shapefile::dbase::FieldValue::Integer(i) => {
                AttributeValue::Number(serde_json::Number::from(i))
            }
            _ => AttributeValue::Null,
        };

        if !matches!(attr_value, AttributeValue::Null) {
            attributes.insert(attr_name, attr_value);
        }
    }

    attributes
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[test]
    fn test_is_zip_file() {
        let zip_header = Bytes::from(vec![0x50, 0x4B, 0x03, 0x04]);
        assert!(is_zip_file(&zip_header));

        let not_zip = Bytes::from(vec![0x00, 0x00, 0x00, 0x00]);
        assert!(!is_zip_file(&not_zip));
    }

    #[tokio::test]
    async fn test_read_shapefile_error_on_raw_bytes() {
        let content = Bytes::from(vec![0x00, 0x00, 0x00, 0x00]);
        let params = ShapefileReaderParam { encoding: None };
        let (sender, _receiver) = mpsc::channel(10);

        let result = read_shapefile(&content, &params, sender).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Direct shapefile bytes not supported"));
    }

    #[test]
    fn test_convert_point_to_geometry() {
        let point = shapefile::Point { x: 10.0, y: 20.0 };
        let shape = shapefile::Shape::Point(point);
        let geometry = convert_shape_to_geometry(shape).unwrap();

        match geometry.value {
            GeometryValue::CityGmlGeometry(citygml) => {
                assert_eq!(citygml.gml_geometries.len(), 1);
                assert_eq!(citygml.gml_geometries[0].ty, GeometryType::Point);
                assert_eq!(citygml.gml_geometries[0].line_strings.len(), 1);
            }
            _ => panic!("Expected CityGmlGeometry"),
        }
    }

    #[test]
    fn test_convert_record_to_attributes() {
        use shapefile::dbase::FieldValue;

        let mut record = shapefile::dbase::Record::default();
        record.insert(
            "name".to_string(),
            FieldValue::Character(Some("Test".to_string())),
        );
        record.insert("value".to_string(), FieldValue::Integer(42));
        record.insert("flag".to_string(), FieldValue::Logical(Some(true)));

        let attributes = convert_record_to_attributes(record);
        assert_eq!(attributes.len(), 3);
        assert_eq!(
            attributes.get(&Attribute::new("name")),
            Some(&AttributeValue::String("Test".to_string()))
        );
    }
}
