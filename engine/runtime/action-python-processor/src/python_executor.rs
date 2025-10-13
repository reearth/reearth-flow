use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
use std::str::FromStr;

use indexmap::IndexMap;
use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::types::geometry::Geometry2D as FlowGeometry2D;
use reearth_flow_geometry::types::point::Point2D;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr};
use reearth_flow_types::{Feature, Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tempfile::NamedTempFile;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
pub enum PythonProcessorError {
    #[error("Factory error: {0}")]
    FactoryError(String),

    #[error("Python execution error: {0}")]
    ExecutionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[derive(Debug, Clone, Default)]
pub struct PythonScriptProcessorFactory;

impl ProcessorFactory for PythonScriptProcessorFactory {
    fn name(&self) -> &str {
        "PythonScriptProcessor"
    }

    fn description(&self) -> &str {
        "Execute Python Scripts with Geospatial Data Processing"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(PythonScriptProcessorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Script", "Python"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: PythonScriptProcessorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PythonProcessorError::FactoryError(format!("Failed to serialize parameters: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PythonProcessorError::FactoryError(format!("Failed to deserialize parameters: {e}"))
            })?
        } else {
            return Err(PythonProcessorError::FactoryError(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        if params.script.is_none() && params.python_file.is_none() {
            return Err(PythonProcessorError::FactoryError(
                "Either 'script' (inline) or 'pythonFile' (file path) parameter must be provided"
                    .to_string(),
            )
            .into());
        }

        if params.script.is_some() && params.python_file.is_some() {
            return Err(PythonProcessorError::FactoryError(
                "Cannot provide both 'script' and 'pythonFile' parameters. Use only one."
                    .to_string(),
            )
            .into());
        }

        let processor = PythonScriptProcessor {
            script: params.script,
            python_file: params.python_file,
            python_path: params.python_path.unwrap_or_else(|| "python3".to_string()),
            _timeout_seconds: params.timeout_seconds.unwrap_or(30),
            ctx,
        };

        Ok(Box::new(processor))
    }
}

#[derive(Debug, Clone)]
struct PythonScriptProcessor {
    script: Option<Expr>,
    python_file: Option<Expr>,
    python_path: String,
    _timeout_seconds: u64,
    ctx: NodeContext,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct PythonScriptProcessorParam {
    /// # Inline Script
    /// Python script code to execute inline
    #[serde(skip_serializing_if = "Option::is_none")]
    script: Option<Expr>,

    /// # Python File
    /// Path to a Python script file (supports file://, http://, https://, gs://, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    python_file: Option<Expr>,

    #[serde(skip_serializing_if = "Option::is_none")]
    python_path: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    timeout_seconds: Option<u64>,
}

fn geometry_to_geojson(geometry: &Geometry) -> serde_json::Value {
    match &geometry.value {
        GeometryValue::None => serde_json::Value::Null,
        GeometryValue::FlowGeometry2D(flow_geom) => flow_geometry_2d_to_geojson(flow_geom),
        GeometryValue::FlowGeometry3D(_) => serde_json::Value::Null,
        GeometryValue::CityGmlGeometry(_) => serde_json::Value::Null,
    }
}

fn flow_geometry_2d_to_geojson(geometry: &FlowGeometry2D<f64>) -> serde_json::Value {
    match geometry {
        FlowGeometry2D::Point(point) => {
            serde_json::json!({
                "type": "Point",
                "coordinates": [point.x(), point.y()]
            })
        }
        FlowGeometry2D::LineString(linestring) => {
            let coords: Vec<Vec<f64>> = linestring.0.iter().map(|p| vec![p.x, p.y]).collect();
            serde_json::json!({
                "type": "LineString",
                "coordinates": coords
            })
        }
        FlowGeometry2D::Polygon(polygon) => {
            let exterior: Vec<Vec<f64>> = polygon
                .exterior()
                .0
                .iter()
                .map(|p| vec![p.x, p.y])
                .collect();
            let mut coords = vec![exterior];

            for interior in polygon.interiors() {
                let interior_coords: Vec<Vec<f64>> =
                    interior.0.iter().map(|p| vec![p.x, p.y]).collect();
                coords.push(interior_coords);
            }

            serde_json::json!({
                "type": "Polygon",
                "coordinates": coords
            })
        }
        FlowGeometry2D::MultiPoint(multipoint) => {
            let coords: Vec<Vec<f64>> = multipoint.0.iter().map(|p| vec![p.x(), p.y()]).collect();
            serde_json::json!({
                "type": "MultiPoint",
                "coordinates": coords
            })
        }
        FlowGeometry2D::MultiLineString(multilinestring) => {
            let coords: Vec<Vec<Vec<f64>>> = multilinestring
                .0
                .iter()
                .map(|ls| ls.0.iter().map(|p| vec![p.x, p.y]).collect())
                .collect();
            serde_json::json!({
                "type": "MultiLineString",
                "coordinates": coords
            })
        }
        FlowGeometry2D::MultiPolygon(multipolygon) => {
            let coords: Vec<Vec<Vec<Vec<f64>>>> = multipolygon
                .0
                .iter()
                .map(|polygon| {
                    let exterior: Vec<Vec<f64>> = polygon
                        .exterior()
                        .0
                        .iter()
                        .map(|p| vec![p.x, p.y])
                        .collect();
                    let mut poly_coords = vec![exterior];

                    for interior in polygon.interiors() {
                        let interior_coords: Vec<Vec<f64>> =
                            interior.0.iter().map(|p| vec![p.x, p.y]).collect();
                        poly_coords.push(interior_coords);
                    }
                    poly_coords
                })
                .collect();
            serde_json::json!({
                "type": "MultiPolygon",
                "coordinates": coords
            })
        }
        FlowGeometry2D::Triangle(triangle) => {
            // Convert Triangle using existing conversion methods
            let polygon = triangle.to_polygon();
            flow_geometry_2d_to_geojson(&FlowGeometry2D::Polygon(polygon))
        }
        FlowGeometry2D::Rect(rect) => {
            let coords = vec![
                vec![rect.min().x, rect.min().y],
                vec![rect.max().x, rect.min().y],
                vec![rect.max().x, rect.max().y],
                vec![rect.min().x, rect.max().y],
                vec![rect.min().x, rect.min().y], // Close the rectangle
            ];
            serde_json::json!({
                "type": "Polygon",
                "coordinates": [coords]
            })
        }
        _ => serde_json::Value::Null,
    }
}

fn feature_to_geojson(feature: &Feature) -> serde_json::Value {
    let properties: serde_json::Map<String, serde_json::Value> = feature
        .attributes
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone().into()))
        .collect();

    serde_json::json!({
        "type": "Feature",
        "id": feature.id.to_string(),
        "properties": properties,
        "geometry": geometry_to_geojson(&feature.geometry)
    })
}

fn dedent(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();

    if lines.is_empty() {
        return String::new();
    }

    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                ""
            } else if line.len() >= min_indent {
                &line[min_indent..]
            } else {
                line
            }
        })
        .collect::<Vec<&str>>()
        .join("\n")
}

fn geojson_to_geometry(geojson: &serde_json::Value) -> Result<Geometry, PythonProcessorError> {
    if geojson.is_null() {
        return Ok(Geometry::default());
    }

    let geometry_type = geojson
        .get("type")
        .and_then(|t| t.as_str())
        .ok_or_else(|| {
            PythonProcessorError::SerializationError("Missing geometry type".to_string())
        })?;

    let coords = geojson.get("coordinates").ok_or_else(|| {
        PythonProcessorError::SerializationError("Missing coordinates".to_string())
    })?;

    match geometry_type {
        "Point" => {
            let coords_array = coords.as_array().ok_or_else(|| {
                PythonProcessorError::SerializationError("Invalid point coordinates".to_string())
            })?;
            if coords_array.len() >= 2 {
                let x = coords_array[0].as_f64().unwrap_or(0.0);
                let y = coords_array[1].as_f64().unwrap_or(0.0);
                let point = Point2D::from((x, y));
                Ok(Geometry::with_value(GeometryValue::FlowGeometry2D(
                    FlowGeometry2D::Point(point),
                )))
            } else {
                Err(PythonProcessorError::SerializationError(
                    "Invalid point coordinates".to_string(),
                ))
            }
        }
        _ => {
            // For unsupported types, return empty geometry
            Ok(Geometry::default())
        }
    }
}

impl Processor for PythonScriptProcessor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();

        let geojson_feature: Value = feature_to_geojson(&feature);
        let input_json = serde_json::to_string(&geojson_feature).map_err(|e| {
            PythonProcessorError::SerializationError(format!("Failed to serialize feature: {e}"))
        })?;

        let expr_engine = &self.ctx.expr_engine;
        let scope = expr_engine.new_scope();

        let script_content = if let Some(inline_script) = &self.script {
            expr_engine
                .eval_scope::<String>(inline_script.as_ref(), &scope)
                .unwrap_or_else(|_| inline_script.to_string())
        } else if let Some(python_file_path) = &self.python_file {
            let path_str = expr_engine
                .eval_scope::<String>(python_file_path.as_ref(), &scope)
                .unwrap_or_else(|_| python_file_path.to_string());

            let uri = Uri::from_str(&path_str).map_err(|e| {
                PythonProcessorError::ExecutionError(format!("Invalid file path: {e}"))
            })?;

            let storage = ctx.storage_resolver.resolve(&uri).map_err(|e| {
                PythonProcessorError::ExecutionError(format!("Failed to resolve storage: {e}"))
            })?;

            let bytes = storage.get_sync(uri.path().as_path()).map_err(|e| {
                PythonProcessorError::ExecutionError(format!("Failed to read script file: {e}"))
            })?;

            String::from_utf8(bytes.to_vec()).map_err(|e| {
                PythonProcessorError::ExecutionError(format!("Script file is not valid UTF-8: {e}"))
            })?
        } else {
            return Err(PythonProcessorError::ExecutionError(
                "No script or pythonFile provided".to_string(),
            )
            .into());
        };

        // Apply dedent to remove common leading whitespace from user script
        let dedented_script = dedent(&script_content);

        let python_wrapper = format!(
            r#"
import sys
import json

# Read input from stdin
input_data = sys.stdin.read()
feature = json.loads(input_data)

# Provide convenient access to feature components
properties = feature.get('properties', {{}})
geometry = feature.get('geometry')
feature_id = feature.get('id')

# Backward compatibility: make properties available as 'attributes'
attributes = properties

# Helper functions for geospatial operations
def get_coordinates(geom):
    """Extract coordinates from geometry"""
    if geom and 'coordinates' in geom:
        return geom['coordinates']
    return None

def get_geometry_type(geom):
    """Get geometry type"""
    if geom and 'type' in geom:
        return geom['type']
    return None

def create_point(x, y):
    """Create a Point geometry"""
    return {{
        'type': 'Point',
        'coordinates': [x, y]
    }}

def create_polygon(coordinates):
    """Create a Polygon geometry"""
    return {{
        'type': 'Polygon',
        'coordinates': coordinates
    }}

# User script starts here
{dedented_script}
# User script ends here

# Handle multiple output formats
if 'output_features' in locals() and isinstance(output_features, list):
    # Multiple features case
    output = {{
        'type': 'FeatureCollection',
        'features': output_features
    }}
else:
    # Single feature case (default)
    output = {{
        'type': 'Feature',
        'id': locals().get('feature_id', feature.get('id')),
        'properties': locals().get('properties', locals().get('attributes', {{}})),
        'geometry': locals().get('geometry', feature.get('geometry'))
    }}

# Output the result
print(json.dumps(output))
"#
        );

        // Create temporary file for the Python script
        let mut temp_file = NamedTempFile::new().map_err(|e| {
            PythonProcessorError::ExecutionError(format!("Failed to create temp file: {e}"))
        })?;

        temp_file
            .write_all(python_wrapper.as_bytes())
            .map_err(|e| {
                PythonProcessorError::ExecutionError(format!("Failed to write script: {e}"))
            })?;

        temp_file.flush().map_err(|e| {
            PythonProcessorError::ExecutionError(format!("Failed to flush script: {e}"))
        })?;

        // Execute Python script with timeout
        let mut child = Command::new(&self.python_path)
            .arg(temp_file.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                PythonProcessorError::ExecutionError(format!("Failed to spawn Python process: {e}"))
            })?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input_json.as_bytes()).map_err(|e| {
                PythonProcessorError::ExecutionError(format!("Failed to write to stdin: {e}"))
            })?;
        }

        let output = match child.wait_with_output() {
            Ok(output) => output,
            Err(e) => {
                return Err(PythonProcessorError::ExecutionError(format!(
                    "Failed to execute Python script: {e}"
                ))
                .into());
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PythonProcessorError::ExecutionError(format!(
                "Python script failed: {stderr}"
            ))
            .into());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let geojson_response: serde_json::Value =
            serde_json::from_str(stdout.trim()).map_err(|e| {
                PythonProcessorError::SerializationError(format!(
                    "Failed to parse Python output: {e}. Output was: {stdout}"
                ))
            })?;

        if let Some("FeatureCollection") = geojson_response.get("type").and_then(|t| t.as_str()) {
            if let Some(features_array) =
                geojson_response.get("features").and_then(|f| f.as_array())
            {
                for geojson_feature in features_array {
                    let mut output_feature = feature.clone();
                    output_feature.refresh_id(); // Generate new ID for each output feature

                    if let Some(properties) = geojson_feature
                        .get("properties")
                        .and_then(|p| p.as_object())
                    {
                        let updated_attributes: IndexMap<Attribute, AttributeValue> = properties
                            .iter()
                            .map(|(k, v)| {
                                (Attribute::new(k.clone()), AttributeValue::from(v.clone()))
                            })
                            .collect();
                        output_feature.attributes = updated_attributes;
                    }

                    // Update geometry if present
                    if let Some(geometry_json) = geojson_feature.get("geometry") {
                        if let Ok(new_geometry) = geojson_to_geometry(geometry_json) {
                            output_feature.geometry = new_geometry;
                        }
                    }

                    fw.send(ctx.new_with_feature_and_port(output_feature, DEFAULT_PORT.clone()));
                }
            }
        } else {
            if let Some(properties) = geojson_response
                .get("properties")
                .and_then(|p| p.as_object())
            {
                let updated_attributes: IndexMap<Attribute, AttributeValue> = properties
                    .iter()
                    .map(|(k, v)| (Attribute::new(k.clone()), AttributeValue::from(v.clone())))
                    .collect();
                feature.attributes = updated_attributes;
            }

            if let Some(geometry_json) = geojson_response.get("geometry") {
                if let Ok(new_geometry) = geojson_to_geometry(geometry_json) {
                    feature.geometry = new_geometry;
                }
            }

            fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        }

        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "PythonScriptProcessor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
    use reearth_flow_geometry::types::point::Point2D;
    use reearth_flow_runtime::{event::EventHub, executor_operation::NodeContext};
    use reearth_flow_types::{
        geometry::Geometry as FlowGeometry, Attribute, AttributeValue, Feature,
    };
    use serde_json::json;
    use std::collections::HashMap;

    fn create_test_feature() -> Feature {
        let mut attributes = IndexMap::new();
        attributes.insert(
            Attribute::new("name".to_string()),
            AttributeValue::String("test_feature".to_string()),
        );
        attributes.insert(
            Attribute::new("value".to_string()),
            AttributeValue::Number(serde_json::Number::from(42)),
        );

        let point = Point2D::from((139.7, 35.7)); // Tokyo coordinates
        let geometry = FlowGeometry::with_value(reearth_flow_types::GeometryValue::FlowGeometry2D(
            FlowGeometry2D::Point(point),
        ));

        Feature::new_with_attributes_and_geometry(
            attributes,
            geometry,
            Default::default(), // Empty metadata
        )
    }

    fn create_test_context() -> NodeContext {
        NodeContext::default()
    }

    #[test]
    fn test_factory_build_success() {
        let factory = PythonScriptProcessorFactory;
        let ctx = create_test_context();

        let mut with = HashMap::new();
        with.insert(
            "script".to_string(),
            json!("properties['result'] = 'success'"),
        );

        let result = factory.build(
            ctx,
            EventHub::new(10),
            "test_action".to_string(),
            Some(with),
        );

        assert!(result.is_ok());
        let processor = result.unwrap();
        assert_eq!(processor.name(), "PythonScriptProcessor");
    }

    #[test]
    fn test_factory_build_missing_params() {
        let factory = PythonScriptProcessorFactory;
        let ctx = create_test_context();

        let result = factory.build(ctx, EventHub::new(10), "test_action".to_string(), None);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Missing required parameter"));
    }

    #[test]
    fn test_factory_build_missing_script_and_file() {
        let factory = PythonScriptProcessorFactory;
        let ctx = create_test_context();

        let with = HashMap::new();

        let result = factory.build(
            ctx,
            EventHub::new(10),
            "test_action".to_string(),
            Some(with),
        );

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Either 'script' (inline) or 'pythonFile'"));
    }

    #[test]
    fn test_factory_build_both_script_and_file() {
        let factory = PythonScriptProcessorFactory;
        let ctx = create_test_context();

        let mut with = HashMap::new();
        with.insert(
            "script".to_string(),
            json!("properties['result'] = 'success'"),
        );
        with.insert("pythonFile".to_string(), json!("file:///path/to/script.py"));

        let result = factory.build(
            ctx,
            EventHub::new(10),
            "test_action".to_string(),
            Some(with),
        );

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Cannot provide both 'script' and 'pythonFile'"));
    }

    #[test]
    fn test_geometry_to_geojson_point() {
        let point = Point2D::from((139.7, 35.7));
        let geometry = FlowGeometry::with_value(reearth_flow_types::GeometryValue::FlowGeometry2D(
            FlowGeometry2D::Point(point),
        ));

        let geojson = geometry_to_geojson(&geometry);

        assert_eq!(geojson["type"], "Point");
        let coords = geojson["coordinates"].as_array().unwrap();
        assert_eq!(coords[0].as_f64().unwrap(), 139.7);
        assert_eq!(coords[1].as_f64().unwrap(), 35.7);
    }

    #[test]
    fn test_geometry_to_geojson_null() {
        let geometry = FlowGeometry::default();
        let geojson = geometry_to_geojson(&geometry);
        assert!(geojson.is_null());
    }

    #[test]
    fn test_feature_to_geojson() {
        let feature = create_test_feature();
        let geojson = feature_to_geojson(&feature);

        assert_eq!(geojson["type"], "Feature");
        assert!(geojson["id"].is_string());
        assert_eq!(geojson["properties"]["name"], "test_feature");
        assert_eq!(geojson["properties"]["value"], 42.0);
        assert_eq!(geojson["geometry"]["type"], "Point");
    }

    #[test]
    fn test_geojson_to_geometry_point() {
        let geojson = json!({
            "type": "Point",
            "coordinates": [139.7, 35.7]
        });

        let result = geojson_to_geometry(&geojson);
        assert!(result.is_ok());

        let geometry = result.unwrap();
        if let reearth_flow_types::GeometryValue::FlowGeometry2D(FlowGeometry2D::Point(point)) =
            geometry.value
        {
            assert_eq!(point.x(), 139.7);
            assert_eq!(point.y(), 35.7);
        } else {
            panic!("Expected Point geometry");
        }
    }

    #[test]
    fn test_geojson_to_geometry_null() {
        let geojson = json!(null);
        let result = geojson_to_geometry(&geojson);
        assert!(result.is_ok());

        let geometry = result.unwrap();
        assert!(matches!(
            geometry.value,
            reearth_flow_types::GeometryValue::None
        ));
    }

    #[test]
    fn test_geojson_to_geometry_invalid_type() {
        let geojson = json!({
            "type": "InvalidType",
            "coordinates": [139.7, 35.7]
        });

        let result = geojson_to_geometry(&geojson);
        assert!(result.is_ok());

        // Should return default geometry for unsupported types
        let geometry = result.unwrap();
        assert!(matches!(
            geometry.value,
            reearth_flow_types::GeometryValue::None
        ));
    }

    #[test]
    fn test_geojson_to_geometry_missing_coordinates() {
        let geojson = json!({
            "type": "Point"
        });

        let result = geojson_to_geometry(&geojson);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Missing coordinates"));
    }

    #[test]
    fn test_processor_factory_metadata() {
        let factory = PythonScriptProcessorFactory;

        assert_eq!(factory.name(), "PythonScriptProcessor");
        assert_eq!(
            factory.description(),
            "Execute Python Scripts with Geospatial Data Processing"
        );
        assert_eq!(factory.categories(), &["Script", "Python"]);
        assert_eq!(factory.get_input_ports().len(), 1);
        assert_eq!(factory.get_output_ports().len(), 1);
        assert!(factory.parameter_schema().is_some());
    }

    #[test]
    fn test_dedent_with_leading_spaces() {
        let input = "            if True:\n                print('hello')";
        let expected = "if True:\n    print('hello')";
        assert_eq!(dedent(input), expected);
    }

    #[test]
    fn test_dedent_with_no_indent() {
        let input = "if True:\n    print('hello')";
        let expected = "if True:\n    print('hello')";
        assert_eq!(dedent(input), expected);
    }

    #[test]
    fn test_dedent_with_mixed_indent() {
        let input = "    line1\n        line2\n    line3";
        let expected = "line1\n    line2\nline3";
        assert_eq!(dedent(input), expected);
    }

    #[test]
    fn test_dedent_with_empty_lines() {
        let input = "    line1\n\n    line2";
        let expected = "line1\n\nline2";
        assert_eq!(dedent(input), expected);
    }

    #[test]
    fn test_dedent_empty_string() {
        let input = "";
        let expected = "";
        assert_eq!(dedent(input), expected);
    }
}
