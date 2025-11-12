use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use approx::AbsDiffEq;
use quick_xml::events::Event;
use quick_xml::Reader;
use reearth_flow_geometry::types::{
    coordinate::Coordinate3D, geometry::Geometry3D, line_string::LineString3D, point::Point3D, triangular_mesh::TriangularMesh
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Geometry as FlowGeometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::PlateauProcessorError;

#[derive(Debug, Clone, Default)]
pub struct CityGmlMeshBuilderFactory;

impl ProcessorFactory for CityGmlMeshBuilderFactory {
    fn name(&self) -> &str {
        "PLATEAU4.CityGmlMeshBuilder"
    }

    fn description(&self) -> &str {
        "Validates CityGML mesh triangles by parsing raw XML: (1) each triangle has exactly 4 vertices, (2) each triangle is closed (first vertex equals last vertex)"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CityGmlMeshBuilderParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU", "Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            DEFAULT_PORT.clone(),
            Port::new("not_closed"),
            Port::new("incorrect_vertices"),
            Port::new("wrong_orientation"),
            Port::new("degenerate_triangle"),
            Port::new("summary"),
            REJECTED_PORT.clone(),
        ]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: CityGmlMeshBuilderParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::CityGmlMeshBuilderFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::CityGmlMeshBuilderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            CityGmlMeshBuilderParam::default()
        };

        Ok(Box::new(CityGmlMeshBuilder {
            params,
            relief_feature_counter: 0,
        }))
    }
}

/// # CityGML Mesh Builder Parameters
/// Configure validation rules for CityGML mesh triangles
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CityGmlMeshBuilderParam {
    /// # Error Attribute Name
    /// Attribute name to store validation error messages (default: "_validation_error")
    #[serde(default = "default_error_attr")]
    pub error_attribute: Attribute,

    /// # Reject Invalid Features
    /// If true, send invalid features to rejected port; if false, send all features to default port with error attributes
    #[serde(default = "default_reject_invalid")]
    pub reject_invalid: bool,
}

fn default_error_attr() -> Attribute {
    Attribute::new("_validation_error")
}

fn default_reject_invalid() -> bool {
    false
}

impl Default for CityGmlMeshBuilderParam {
    fn default() -> Self {
        Self {
            error_attribute: default_error_attr(),
            reject_invalid: default_reject_invalid(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CityGmlMeshBuilder {
    params: CityGmlMeshBuilderParam,
    relief_feature_counter: u64,
}

impl Processor for CityGmlMeshBuilder {
    fn num_threads(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();

        // Get the CityGML file path from feature attributes
        let file_path = match feature.attributes.get(&Attribute::new("path")) {
            Some(AttributeValue::String(path)) => {
                // Remove file:// prefix if present
                path.strip_prefix("file://").unwrap_or(path).to_string()
            }
            _ => {
                // No path attribute, pass through unchanged
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
                // Also send summary feature for consistency
                let mut summary_feature = feature.clone();
                summary_feature.geometry = FlowGeometry::default();
                summary_feature.attributes.insert(
                    Attribute::new("_relief_index"),
                    AttributeValue::Number(self.relief_feature_counter.into()),
                );
                fw.send(ctx.new_with_feature_and_port(summary_feature, Port::new("summary")));
                return Ok(());
            }
        };

        // Parse the raw XML and validate triangles
        let validation_result = match self.parse_and_validate_triangles(&file_path) {
            Ok(result) => result,
            Err(e) => {
                // If we can't parse the file, pass through with error
                feature.attributes.insert(
                    self.params.error_attribute.clone(),
                    AttributeValue::String(format!("Failed to parse CityGML: {e}")),
                );
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
                // Also send summary feature for consistency
                let mut summary_feature = feature.clone();
                summary_feature.geometry = FlowGeometry::default();
                summary_feature.attributes.insert(
                    Attribute::new("_relief_index"),
                    AttributeValue::Number(self.relief_feature_counter.into()),
                );
                fw.send(ctx.new_with_feature_and_port(summary_feature, Port::new("summary")));
                return Ok(());
            }
        };

        // Add the Relief Index attribute
        feature.attributes.insert(
            Attribute::new("_relief_index"),
            AttributeValue::Number(self.relief_feature_counter.into()),
        );

        let ValidationResult { errors, triangular_mesh, epsg_code } = validation_result;
        if let Some(triangular_mesh) = triangular_mesh {
            let geometry_value = GeometryValue::FlowGeometry3D(Geometry3D::TriangularMesh(
                triangular_mesh,
            ));
            let geometry = FlowGeometry {
                epsg: epsg_code,
                value: geometry_value,
            };
            feature.geometry = geometry;

            // Send to default port
            fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            return Ok(());
        }

        // Create and send summary feature
        // This contains file metadata with a null geometry for aggregation
        let mut summary_feature = feature.clone();
        summary_feature.geometry = FlowGeometry::default(); // Null geometry
        fw.send(ctx.new_with_feature_and_port(summary_feature, Port::new("summary")));

        // Handle validation results - route to appropriate ports
        for error in errors {
            let mut error_feature = feature.clone();
            let Error { error_type, geometry } = error;
            error_feature.geometry = FlowGeometry { epsg: epsg_code, value: geometry };
            let port_name = match error_type {
                ErrorType::NotClosed => "not_closed",
                ErrorType::IncorrectNumVertices => "incorrect_vertices",
                ErrorType::WrongOrientation => "wrong_orientation",
                ErrorType::DegenerateTriangle => "degenerate_triangle",
            };
            fw.send(ctx.new_with_feature_and_port(
                error_feature,
                Port::new(port_name),
            ));
        }

        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "CityGmlMeshBuilder"
    }
}

/// Validation result with categorized errors and geometry data
#[derive(Debug)]
struct ValidationResult {
    errors: Vec<Error>,
    // Geometry data
    triangular_mesh: Option<TriangularMesh<f64>>,
    epsg_code: Option<nusamai_projection::crs::EpsgCode>,
}

#[derive(Debug)]
struct Error {
    pub error_type: ErrorType,
    pub geometry: GeometryValue
}

#[derive(Debug)]
enum ErrorType {
    NotClosed,
    IncorrectNumVertices,
    WrongOrientation,
    DegenerateTriangle,
}

impl CityGmlMeshBuilder {
    /// Parse CityGML XML and validate triangle coordinates before polygon construction
    fn parse_and_validate_triangles(
        &mut self,
        file_path: &str,
    ) -> Result<ValidationResult, Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let buf_reader = BufReader::new(file);
        let mut reader = Reader::from_reader(buf_reader);
        reader.config_mut().trim_text(true);

        let mut errors = Vec::new();
        let mut buf = Vec::new();
        let mut inside_triangle = false;
        let mut inside_pos_list = false;
        let mut epsg_code: Option<nusamai_projection::crs::EpsgCode> = None;

        // Collect triangle faces for TriangularMesh
        let mut valid_faces: Vec<[Coordinate3D<f64>; 3]> = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();

                    // Try to extract EPSG code from srsName attribute if not already found
                    if epsg_code.is_none() {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"srsName" {
                                if let Ok(srs_name) = std::str::from_utf8(&attr.value) {
                                    // Parse EPSG code from URN format like "http://www.opengis.net/def/crs/EPSG/0/6697"
                                    if let Some(epsg_str) = srs_name.rsplit('/').next() {
                                        if let Ok(epsg_num) = epsg_str.parse::<u16>() {
                                            epsg_code = Some(epsg_num);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if name.as_ref() == b"gml:Triangle" {
                        inside_triangle = true;
                    } else if inside_triangle && name.as_ref() == b"gml:posList" {
                        inside_pos_list = true;
                    } else if name.as_ref() == b"dem:ReliefFeature" {
                        self.relief_feature_counter += 1;
                    }
                }
                Ok(Event::Text(e)) if inside_pos_list => {
                    // Parse the coordinate list
                    let text = e.unescape()?;
                    let coords_text = text.trim();

                    // Split by whitespace and parse as f64
                    let values: Vec<f64> = coords_text
                        .split_whitespace()
                        .filter_map(|s| s.parse::<f64>().ok())
                        .collect();

                    // Values come in groups of 3 (x, y, z)
                    if values.len() % 3 != 0 {
                        let error = Error {
                            error_type: ErrorType::IncorrectNumVertices,
                            geometry: GeometryValue::FlowGeometry3D(Geometry3D::Point(Point3D::new(
                                values[0],
                                values[1],
                                values[2],
                            ))),
                        };
                        errors.push(error);
                    } else {
                        let num_vertices = values.len() / 3;
                        let mut has_error = false;

                        // Build coordinates for the triangle
                        let mut coords = Vec::new();
                        for i in 0..num_vertices {
                            let x = values[i * 3];
                            let y = values[i * 3 + 1];
                            let z = values[i * 3 + 2];
                            coords.push(Coordinate3D::new__(x, y, z));
                        }

                        // Validation 1: Check vertex count
                        if num_vertices != 4 {
                            has_error = true;
                            let error = Error {
                                error_type: ErrorType::IncorrectNumVertices,
                                geometry: GeometryValue::FlowGeometry3D(Geometry3D::LineString(LineString3D::new(
                                    coords.clone()
                                )))
                            };
                            errors.push(error);
                        } else {
                            // Validation 2: Check if triangle is closed (first == last)
                            let first_x = values[0];
                            let first_y = values[1];
                            let first_z = values[2];
                            let last_x = values[values.len() - 3];
                            let last_y = values[values.len() - 2];
                            let last_z = values[values.len() - 1];

                            const EPSILON: f64 = 1e-10;
                            let is_closed = (first_x - last_x).abs() < EPSILON
                                && (first_y - last_y).abs() < EPSILON
                                && (first_z - last_z).abs() < EPSILON;

                            if !is_closed {
                                has_error = true;
                                let error = Error {
                                    error_type: ErrorType::NotClosed,
                                    geometry: GeometryValue::FlowGeometry3D(Geometry3D::LineString(LineString3D::new(
                                        coords.clone()
                                    )))
                                };
                                errors.push(error);
                            }

                            // Validation 3: Check triangle orientation via normal vector
                            if !has_error && coords.len() == 4 {
                                let v0 = coords[0];
                                let v1 = coords[1];
                                let v2 = coords[2];

                                // Edge vectors
                                let edge1_x = v1.x - v0.x;
                                let edge1_y = v1.y - v0.y;

                                let edge2_x = v2.x - v0.x;
                                let edge2_y = v2.y - v0.y;

                                // Cross product: edge1 × edge2
                                let normal_z = edge1_x * edge2_y - edge1_y * edge2_x;

                                // Check if z-component is negative
                                if normal_z > 0.0 {
                                    has_error = true;
                                    let error = Error {
                                        error_type: ErrorType::WrongOrientation,
                                        geometry: GeometryValue::FlowGeometry3D(Geometry3D::LineString(LineString3D::new(
                                            coords.clone()
                                        )))
                                    };
                                    errors.push(error);
                                }
                            }

                            // Validation 4: Check for degenerate triangle (area zero)
                            let a = coords[0];
                            let b = coords[1];
                            let c = coords[2];

                            let (a, b, c) = {
                                let mean = (a + b + c) / 3.0;
                                let a = a - mean;
                                let b = b - mean;
                                let c = c - mean;
                                (a.normalize(), b.normalize(), c.normalize())
                            };

                            let ab = (b - a).norm();
                            let ac = (c - a).norm();
                            let bc = (c - b).norm();
                            let epsilon: f64 = (ab + bc + ac) / (3.0 * 1e5);
                            let is_degenerate = (ab + ac).abs_diff_eq(&bc, epsilon)
                                || (ab + bc).abs_diff_eq(&ac, epsilon)
                                || (ac + bc).abs_diff_eq(&ab, epsilon);
                            if is_degenerate {
                                has_error = true;
                                let error = Error {
                                    error_type: ErrorType::DegenerateTriangle,
                                    geometry: GeometryValue::FlowGeometry3D(Geometry3D::LineString(LineString3D::new(
                                        coords.clone()
                                    )))
                                };
                                errors.push(error);
                            }
                        }

                        // If no errors, add this triangle as a valid face
                        if !has_error && coords.len() == 4 {
                            valid_faces.push([coords[0], coords[1], coords[2]]);
                        }
                    }

                    inside_pos_list = false;
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    if name.as_ref() == b"gml:Triangle" {
                        inside_triangle = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(format!(
                        "XML parse error at position {}: {}",
                        reader.buffer_position(),
                        e
                    )
                    .into())
                }
                _ => {}
            }
            buf.clear();
        }

        // Build the TriangularMesh if we have valid faces and no errors
        let triangular_mesh = if !valid_faces.is_empty() && errors.is_empty() {
            Some(TriangularMesh::from_triangles(valid_faces))
        } else {
            None
        };

        Ok(ValidationResult {
            errors,
            triangular_mesh,
            epsg_code,
        })
    }
}
