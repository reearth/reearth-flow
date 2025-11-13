use super::errors::PlateauProcessorError;
use nusamai_citygml::GML31_NS;
use once_cell::sync::Lazy;
use reearth_flow_common::uri::Uri;
use reearth_flow_common::xml;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{Context, ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

static ERROR_PORT: Lazy<Port> = Lazy::new(|| Port::new("error"));
static SUMMARY_PORT: Lazy<Port> = Lazy::new(|| Port::new("summary"));
static PASSED_PORT: Lazy<Port> = Lazy::new(|| Port::new("passed"));
static ALL_PORT: Lazy<Port> = Lazy::new(|| Port::new("all"));

// Validation attribute names (shared with unshared_edge_detector)
pub(crate) const ATTR_IS_INCORRECT_NUM_VERTICES: &str = "__is_incorrect_num_vertices";
pub(crate) const ATTR_IS_NOT_CLOSED: &str = "__is_not_closed";
pub(crate) const ATTR_IS_WRONG_ORIENTATION: &str = "__is_wrong_orientation";

#[derive(Debug, Clone, Default)]
pub struct FaceExtractorFactory;

impl ProcessorFactory for FaceExtractorFactory {
    fn name(&self) -> &str {
        "PLATEAU4.FaceExtractor"
    }

    fn description(&self) -> &str {
        "Validates individual surfaces of WaterBody features for TIN mesh quality"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FaceExtractorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            ERROR_PORT.clone(),
            SUMMARY_PORT.clone(),
            PASSED_PORT.clone(),
            ALL_PORT.clone(),
        ]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FaceExtractorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::FaceExtractorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::FaceExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            FaceExtractorParam::default()
        };

        let processor = FaceExtractor {
            params,
            buffer: FileBuffer::new(),
        };
        Ok(Box::new(processor))
    }
}

/// # FaceExtractor Parameters
///
/// Configuration for validating individual surfaces of WaterBody features.
/// Always checks vertex count, closure, and orientation of polygons in TIN meshes.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FaceExtractorParam {
    /// Attribute name for city_gml_path (default: "_gml_path")
    #[serde(default = "default_gml_path_attribute")]
    pub city_gml_path_attribute: Attribute,
}

fn default_gml_path_attribute() -> Attribute {
    Attribute::new("_gml_path")
}

impl Default for FaceExtractorParam {
    fn default() -> Self {
        Self {
            city_gml_path_attribute: Attribute::new("_gml_path"),
        }
    }
}

#[derive(Debug, Clone)]
struct ValidationResult {
    is_incorrect_num_vertices: bool,
    is_not_closed: bool,
    is_wrong_orientation: bool,
}

impl ValidationResult {
    fn has_error(&self) -> bool {
        self.is_incorrect_num_vertices || self.is_not_closed || self.is_wrong_orientation
    }
}

#[derive(Debug, Clone, Default)]
struct FileStatistics {
    num_instances: usize,
    num_incorrect_num_vertices: usize,
    num_not_closed: usize,
    num_wrong_orientation: usize,
}

type BufferResult = (String, FileStatistics, Vec<Feature>, Vec<Feature>, Feature);

#[derive(Debug, Clone, Default)]
struct FileBuffer {
    current_file: Option<String>,
    stats: FileStatistics,
    error_features: Vec<Feature>,
    passed_features: Vec<Feature>,
    base_feature: Option<Feature>,
}

impl FileBuffer {
    fn new() -> Self {
        Self::default()
    }

    fn should_flush(&mut self, file_path: &str) -> bool {
        if let Some(current) = &self.current_file {
            current != file_path
        } else {
            false
        }
    }

    fn set_base_feature(&mut self, file_path: String, feature: Feature) {
        if self.current_file.is_none() {
            self.current_file = Some(file_path);
            self.base_feature = Some(feature);
        }
    }

    fn add_result(&mut self, feature: Feature, result: ValidationResult) {
        self.stats.num_instances += 1;
        if result.is_incorrect_num_vertices {
            self.stats.num_incorrect_num_vertices += 1;
        }
        if result.is_not_closed {
            self.stats.num_not_closed += 1;
        }
        if result.is_wrong_orientation {
            self.stats.num_wrong_orientation += 1;
        }

        if result.has_error() {
            self.error_features.push(feature);
        } else {
            self.passed_features.push(feature);
        }
    }

    fn take(&mut self) -> Option<BufferResult> {
        if let Some(file_path) = self.current_file.take() {
            let stats = std::mem::take(&mut self.stats);
            let errors = std::mem::take(&mut self.error_features);
            let passed = std::mem::take(&mut self.passed_features);
            let base = self.base_feature.take()?;
            Some((file_path, stats, errors, passed, base))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FaceExtractor {
    params: FaceExtractorParam,
    buffer: FileBuffer,
}

impl FaceExtractor {
    fn validate_vertex_count(coords: &[(f64, f64, f64)]) -> bool {
        // Expected vertex count for TIN triangles: 4 (3 vertices + closing point)
        coords.len() == 4
    }

    fn validate_closure(coords: &[(f64, f64, f64)]) -> bool {
        if coords.is_empty() {
            return false;
        }
        let first = coords.first().unwrap();
        let last = coords.last().unwrap();
        first.0 == last.0 && first.1 == last.1 && first.2 == last.2
    }

    fn validate_orientation(coords: &[(f64, f64, f64)]) -> bool {
        if coords.len() < 3 {
            return false;
        }

        // Calculate normal vector using cross product of first two edges
        // For a triangle with vertices A, B, C:
        // v1 = B - A
        // v2 = C - A
        // normal = v1 × v2
        // Left-hand rule (FME_LEFT_HAND_RULE): normal.z > 0

        let p0 = coords[0];
        let p1 = coords[1];
        let p2 = coords[2];

        let v1_x = p1.0 - p0.0;
        let v1_y = p1.1 - p0.1;

        let v2_x = p2.0 - p0.0;
        let v2_y = p2.1 - p0.1;

        // Cross product z component: v1.x * v2.y - v1.y * v2.x
        let normal_z = v1_x * v2_y - v1_y * v2_x;

        // Left-hand rule: normal should point upward (positive z)
        normal_z > 0.0
    }

    fn validate_pos_list(coords: &[(f64, f64, f64)]) -> ValidationResult {
        let mut result = ValidationResult {
            is_incorrect_num_vertices: false,
            is_not_closed: false,
            is_wrong_orientation: false,
        };

        if !Self::validate_vertex_count(coords) {
            result.is_incorrect_num_vertices = true;
        }

        if !Self::validate_closure(coords) {
            result.is_not_closed = true;
        }

        // Only validate orientation for triangles (correct vertex count)
        if !result.is_incorrect_num_vertices && !Self::validate_orientation(coords) {
            result.is_wrong_orientation = true;
        }

        result
    }

    fn parse_pos_list(&self, pos_text: &str) -> Result<Vec<(f64, f64, f64)>, BoxedError> {
        let values: Vec<f64> = pos_text
            .split_whitespace()
            .map(|s| s.parse::<f64>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                PlateauProcessorError::FaceExtractor(format!("Failed to parse coordinate: {e}"))
            })?;

        if values.len() % 3 != 0 {
            return Err(PlateauProcessorError::FaceExtractor(
                "posList length must be multiple of 3".to_string(),
            )
            .into());
        }

        // Parse as (lat, lon, height) and convert to (lon, lat, height)
        // FME code: coords = [(x, y, z) for x, y, z in zip(v[1::3], v[::3], v[2::3])]
        // This means: x=v[1], y=v[0], z=v[2] (swap first two components)
        let coords: Vec<(f64, f64, f64)> = values
            .chunks(3)
            .map(|chunk| (chunk[1], chunk[0], chunk[2]))
            .collect();

        Ok(coords)
    }

    fn process_xml(
        &mut self,
        ctx: &ExecutorContext,
        _fw: &ProcessorChannelForwarder,
        file_path: String,
        xml_content: String,
    ) -> Result<(), BoxedError> {
        // Set the base feature for this file (clone the input feature)
        self.buffer
            .set_base_feature(file_path.clone(), ctx.feature.clone());

        let document = xml::parse(xml_content).map_err(|e| {
            PlateauProcessorError::FaceExtractor(format!("Failed to parse XML: {e}"))
        })?;
        let xml_ctx = xml::create_context(&document).map_err(|e| {
            PlateauProcessorError::FaceExtractor(format!("Failed to create XML context: {e}"))
        })?;
        let root_node = xml::get_root_readonly_node(&document).map_err(|e| {
            PlateauProcessorError::FaceExtractor(format!("Failed to get root node: {e}"))
        })?;

        // Find all WaterBody elements
        let water_bodies = xml::find_readonly_nodes_by_xpath(
            &xml_ctx,
            "//wtr:WaterBody",
            &root_node,
        )
        .map_err(|e| {
            PlateauProcessorError::FaceExtractor(format!("Failed to find WaterBody elements: {e}"))
        })?;

        for water_body_node in water_bodies {
            // Get gml:id
            let gml_id = water_body_node
                .get_attribute_ns(
                    "id",
                    String::from_utf8(GML31_NS.into_inner().to_vec())
                        .map_err(|e| {
                            PlateauProcessorError::FaceExtractor(format!(
                                "Failed to convert namespace: {e}"
                            ))
                        })?
                        .as_str(),
                )
                .unwrap_or_else(|| "unknown".to_string());

            // Find all gml:posList elements under this WaterBody
            let pos_lists =
                xml::find_readonly_nodes_by_xpath(&xml_ctx, ".//gml:posList", &water_body_node)
                    .map_err(|e| {
                        PlateauProcessorError::FaceExtractor(format!(
                            "Failed to find posList elements: {e}"
                        ))
                    })?;

            for pos_list_node in pos_lists {
                // Get posList text content
                let pos_text = pos_list_node.get_content();

                // Parse coordinates
                let coords = self.parse_pos_list(&pos_text)?;

                // Validate
                let result = Self::validate_pos_list(&coords);

                // Create feature for this surface
                let mut feature = ctx.feature.clone();
                feature.refresh_id();

                // Create polygon geometry from coordinates
                use reearth_flow_geometry::types::{
                    coordinate::Coordinate, geometry::Geometry3D, line_string::LineString3D,
                    polygon::Polygon3D,
                };
                use reearth_flow_types::geometry::{Geometry, GeometryValue};

                // Output in standard geographic order: x=lon, y=lat
                // This matches what ExtendedTransverseMercatorProjection.project_forward(lng, lat, z) expects
                let points: Vec<Coordinate> = coords
                    .iter()
                    .map(|(lon, lat, height)| Coordinate {
                        x: *lon,
                        y: *lat,
                        z: *height,
                    })
                    .collect();

                if !points.is_empty() {
                    let polygon = Polygon3D::new(LineString3D::new(points), vec![]);
                    feature.geometry = Geometry {
                        epsg: Some(6697), // JGD2011 geographic
                        value: GeometryValue::FlowGeometry3D(Geometry3D::Polygon(polygon)),
                    };
                }

                feature.attributes.insert(
                    Attribute::new("gml_id"),
                    AttributeValue::String(gml_id.clone()),
                );

                if result.is_incorrect_num_vertices {
                    feature.attributes.insert(
                        Attribute::new(ATTR_IS_INCORRECT_NUM_VERTICES),
                        AttributeValue::Number(serde_json::Number::from(1)),
                    );
                }

                if result.is_not_closed {
                    feature.attributes.insert(
                        Attribute::new(ATTR_IS_NOT_CLOSED),
                        AttributeValue::Number(serde_json::Number::from(1)),
                    );
                }

                if result.is_wrong_orientation {
                    feature.attributes.insert(
                        Attribute::new(ATTR_IS_WRONG_ORIENTATION),
                        AttributeValue::Number(serde_json::Number::from(1)),
                    );
                }

                self.buffer.add_result(feature, result);
            }
        }

        Ok(())
    }

    fn flush_buffer(&mut self, ctx: Context, fw: &ProcessorChannelForwarder) {
        if let Some((file_path, stats, error_features, passed_features, base_feature)) =
            self.buffer.take()
        {
            // Send error features to error port and collect for all port
            let mut all_features = Vec::new();
            for feature in error_features {
                all_features.push(feature.clone());
                fw.send(ctx.as_executor_context(feature, ERROR_PORT.clone()));
            }

            // Send passed features to passed port and collect for all port
            for feature in passed_features {
                all_features.push(feature.clone());
                fw.send(ctx.as_executor_context(feature, PASSED_PORT.clone()));
            }

            // Send all features to all port
            for feature in all_features {
                fw.send(ctx.as_executor_context(feature, ALL_PORT.clone()));
            }

            // Send summary feature
            // Clone the base feature (input feature) and add error count attributes
            let mut summary_feature = base_feature;

            summary_feature.attributes.insert(
                Attribute::new("__is_summary"),
                AttributeValue::Number(serde_json::Number::from(1)),
            );
            summary_feature.attributes.insert(
                Attribute::new("_file_path"),
                AttributeValue::String(file_path),
            );
            summary_feature.attributes.insert(
                Attribute::new("_num_instances"),
                AttributeValue::Number(serde_json::Number::from(stats.num_instances)),
            );
            summary_feature.attributes.insert(
                Attribute::new("_num_incorrect_num_vertices"),
                AttributeValue::Number(serde_json::Number::from(stats.num_incorrect_num_vertices)),
            );
            summary_feature.attributes.insert(
                Attribute::new("_num_not_closed"),
                AttributeValue::Number(serde_json::Number::from(stats.num_not_closed)),
            );
            summary_feature.attributes.insert(
                Attribute::new("_num_wrong_orientation"),
                AttributeValue::Number(serde_json::Number::from(stats.num_wrong_orientation)),
            );

            // Add _json_filename attribute for JSON output
            // Replace / with _ in udxDirs to create flat filename
            if let Some(AttributeValue::String(udx_dirs)) =
                summary_feature.attributes.get(&Attribute::new("udxDirs"))
            {
                let json_filename = format!("summary_{}.json", udx_dirs.replace('/', "_"));
                summary_feature.attributes.insert(
                    Attribute::new("_json_filename"),
                    AttributeValue::String(json_filename),
                );
            }

            fw.send(ctx.as_executor_context(summary_feature, SUMMARY_PORT.clone()));
        }
    }
}

impl Processor for FaceExtractor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        // Get file path from attributes
        let file_path_attr = feature
            .attributes
            .get(&self.params.city_gml_path_attribute)
            .ok_or(PlateauProcessorError::FaceExtractor(
                "cityGmlPath attribute not found".to_string(),
            ))?;

        let file_path_str = file_path_attr.to_string();

        // Flush buffer if processing a different file
        if self.buffer.should_flush(&file_path_str) {
            self.flush_buffer(ctx.as_context(), fw);
        }

        // Parse URI and read file
        let uri = Uri::from_str(&file_path_str).map_err(|e| {
            PlateauProcessorError::FaceExtractor(format!("Failed to parse URI: {e}"))
        })?;

        let storage = ctx.storage_resolver.resolve(&uri).map_err(|e| {
            PlateauProcessorError::FaceExtractor(format!("Failed to resolve storage: {e}"))
        })?;

        let content = storage.get_sync(uri.path().as_path()).map_err(|e| {
            PlateauProcessorError::FaceExtractor(format!("Failed to read file: {e}"))
        })?;

        let xml_content = String::from_utf8(content.to_vec()).map_err(|e| {
            PlateauProcessorError::FaceExtractor(format!("Failed to convert to UTF-8: {e}"))
        })?;

        self.process_xml(&ctx, fw, file_path_str, xml_content)?;

        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        // Cast self to mutable to flush buffer
        // This is safe because finish is called once at the end
        let mut mutable_self = self.clone();
        mutable_self.flush_buffer(ctx.as_context(), fw);
        Ok(())
    }

    fn name(&self) -> &str {
        "FaceExtractor"
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn create_validator() -> FaceExtractor {
        FaceExtractor {
            params: FaceExtractorParam::default(),
            buffer: FileBuffer::new(),
        }
    }

    #[test]
    fn test_parse_pos_list_valid() {
        let validator = create_validator();

        // Test case: 4 vertices (triangle + closing point)
        let input = "35.01 135.01 0.0 35.01 135.02 0.0 35.02 135.01 0.0 35.01 135.01 0.0";
        let result = validator.parse_pos_list(input).unwrap();

        // FME coordinate swap: (lat, lon, height) → (lon, lat, height)
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], (135.01, 35.01, 0.0));
        assert_eq!(result[1], (135.02, 35.01, 0.0));
        assert_eq!(result[2], (135.01, 35.02, 0.0));
        assert_eq!(result[3], (135.01, 35.01, 0.0));
    }

    #[test]
    fn test_parse_pos_list_invalid_length() {
        let validator = create_validator();

        // Not a multiple of 3
        let input = "35.01 135.01";
        let result = validator.parse_pos_list(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_vertex_count() {
        // Valid: 4 vertices
        let coords = vec![
            (0.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
            (0.0, 1.0, 0.0),
            (0.0, 0.0, 0.0),
        ];
        assert!(FaceExtractor::validate_vertex_count(&coords));

        // Invalid: 2 vertices
        let coords = vec![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0)];
        assert!(!FaceExtractor::validate_vertex_count(&coords));

        // Invalid: 3 vertices
        let coords = vec![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0)];
        assert!(!FaceExtractor::validate_vertex_count(&coords));

        // Invalid: 5 vertices
        let coords = vec![
            (0.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
            (1.0, 1.0, 0.0),
            (0.0, 1.0, 0.0),
            (0.0, 0.0, 0.0),
        ];
        assert!(!FaceExtractor::validate_vertex_count(&coords));
    }

    #[test]
    fn test_validate_closure() {
        // Valid: closed
        let coords = vec![
            (0.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
            (0.0, 1.0, 0.0),
            (0.0, 0.0, 0.0),
        ];
        assert!(FaceExtractor::validate_closure(&coords));

        // Invalid: not closed
        let coords = vec![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0)];
        assert!(!FaceExtractor::validate_closure(&coords));

        // Invalid: empty
        let coords = vec![];
        assert!(!FaceExtractor::validate_closure(&coords));
    }

    #[test]
    fn test_validate_orientation() {
        // Valid: counter-clockwise (left-hand rule, normal_z > 0)
        // Triangle vertices in CCW order
        let coords = vec![
            (0.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
            (0.0, 1.0, 0.0),
            (0.0, 0.0, 0.0),
        ];
        assert!(FaceExtractor::validate_orientation(&coords));

        // Invalid: clockwise (right-hand rule, normal_z < 0)
        let coords = vec![
            (0.0, 0.0, 0.0),
            (0.0, 1.0, 0.0),
            (1.0, 0.0, 0.0),
            (0.0, 0.0, 0.0),
        ];
        assert!(!FaceExtractor::validate_orientation(&coords));

        // Invalid: less than 3 vertices
        let coords = vec![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0)];
        assert!(!FaceExtractor::validate_orientation(&coords));
    }

    #[test]
    fn test_validate_pos_list_incorrect_vertex_count() {
        // Case: Incorrect vertex count (2 vertices, not 4)
        // - Vertex count is wrong
        // - Not closed
        // - Orientation is not checked when vertex count is wrong
        let coords = vec![(0.0, 0.0, 0.0), (1.0, 1.0, 0.0)];
        let result = FaceExtractor::validate_pos_list(&coords);

        assert!(result.is_incorrect_num_vertices);
        assert!(result.is_not_closed);
        assert!(!result.is_wrong_orientation); // Orientation is not validated for incorrect vertex count
    }

    #[test]
    fn test_validate_pos_list_wrong_orientation() {
        // Case: Valid vertex count and closed, but wrong orientation (clockwise)
        let coords = vec![
            (0.0, 0.0, 0.0),
            (0.0, 1.0, 0.0),
            (1.0, 0.0, 0.0),
            (0.0, 0.0, 0.0),
        ];
        let result = FaceExtractor::validate_pos_list(&coords);

        assert!(!result.is_incorrect_num_vertices);
        assert!(!result.is_not_closed);
        assert!(result.is_wrong_orientation);
        assert!(result.has_error());
    }

    #[test]
    fn test_validate_pos_list_no_errors() {
        // Case: Valid triangle
        let coords = vec![
            (0.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
            (0.0, 1.0, 0.0),
            (0.0, 0.0, 0.0),
        ];
        let result = FaceExtractor::validate_pos_list(&coords);

        assert!(!result.is_incorrect_num_vertices);
        assert!(!result.is_not_closed);
        assert!(!result.is_wrong_orientation);
        assert!(!result.has_error());
    }
}
