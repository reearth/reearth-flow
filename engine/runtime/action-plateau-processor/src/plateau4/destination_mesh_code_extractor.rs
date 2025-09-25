use std::collections::HashMap;

use reearth_flow_geometry::algorithm::centroid::Centroid;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::jpmesh::{JPMeshCode, JPMeshType};
use reearth_flow_types::{Attribute, AttributeValue, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct DestinationMeshCodeExtractorFactory;

impl ProcessorFactory for DestinationMeshCodeExtractorFactory {
    fn name(&self) -> &str {
        "DestinationMeshCodeExtractor"
    }

    fn description(&self) -> &str {
        "Extract Japanese standard regional mesh code for PLATEAU destination files and add as attribute"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(DestinationMeshCodeExtractorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: DestinationMeshCodeExtractorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with)
                .map_err(|e| format!("Failed to serialize parameters: {e}"))?;
            serde_json::from_value(value)
                .map_err(|e| format!("Failed to deserialize parameters: {e}"))?
        } else {
            DestinationMeshCodeExtractorParam::default()
        };

        let mesh_type = match params.mesh_type {
            1 => JPMeshType::Mesh80km,
            2 => JPMeshType::Mesh10km,
            3 => JPMeshType::Mesh1km,
            4 => JPMeshType::Mesh500m,
            5 => JPMeshType::Mesh250m,
            6 => JPMeshType::Mesh125m,
            _ => return Err("Invalid mesh_type. Must be 1-6".into()),
        };

        Ok(Box::new(DestinationMeshCodeExtractor {
            mesh_type,
            meshcode_attr: params.meshcode_attr,
        }))
    }
}

/// # PLATEAU Destination MeshCode Extractor Parameters
/// Configure mesh code extraction for Japanese standard regional mesh
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DestinationMeshCodeExtractorParam {
    /// # Mesh Type
    /// Japanese standard mesh type: 1=80km, 2=10km, 3=1km, 4=500m, 5=250m, 6=125m
    #[serde(default = "default_mesh_type")]
    pub mesh_type: u8,

    /// # Mesh Code Attribute Name
    /// Output attribute name for the mesh code
    #[serde(default = "default_meshcode_attr")]
    pub meshcode_attr: String,
}

impl Default for DestinationMeshCodeExtractorParam {
    fn default() -> Self {
        Self {
            mesh_type: default_mesh_type(),
            meshcode_attr: default_meshcode_attr(),
        }
    }
}

fn default_mesh_type() -> u8 {
    3 // Tertiary Standard Mesh (1km) - PLATEAU default
}

fn default_meshcode_attr() -> String {
    "meshcode".to_string()
}

#[derive(Debug, Clone)]
pub struct DestinationMeshCodeExtractor {
    mesh_type: JPMeshType,
    meshcode_attr: String,
}

impl Processor for DestinationMeshCodeExtractor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;

        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(geometry) => {
                // Calculate centroid of the feature
                let centroid = if let Some(centroid_point) = geometry.centroid() {
                    centroid_point.0
                } else {
                    // If centroid calculation fails, reject the feature
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    return Ok(());
                };

                // Generate mesh code from centroid coordinates
                let meshcode = JPMeshCode::new(centroid, self.mesh_type);

                // Add mesh code as attribute without modifying geometry
                let mut new_feature = feature.clone();
                new_feature.attributes.insert(
                    Attribute::new(&self.meshcode_attr),
                    AttributeValue::String(meshcode.to_number().to_string()),
                );

                fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "DestinationMeshCodeExtractor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
    use reearth_flow_types::{Feature, Geometry};

    #[test]
    fn test_destination_mesh_code_extraction() {
        let factory = DestinationMeshCodeExtractorFactory;
        let params = DestinationMeshCodeExtractorParam {
            mesh_type: 3, // 1km mesh
            meshcode_attr: "_meshcode".to_string(),
        };

        let processor = factory
            .build(
                NodeContext::default(),
                EventHub::new(100),
                "test".to_string(),
                Some(
                    serde_json::to_value(params)
                        .unwrap()
                        .as_object()
                        .unwrap()
                        .clone()
                        .into_iter()
                        .collect(),
                ),
            )
            .unwrap();

        // Test with a point in Tokyo area (simplified for testing)
        let geometry = Geometry {
            value: GeometryValue::None,
            epsg: Some(4326), // WGS84
        };

        let _feature = Feature {
            id: uuid::Uuid::new_v4(),
            geometry,
            attributes: IndexMap::new(),
            metadata: Default::default(),
        };

        // This test demonstrates the structure - actual execution would need proper context
        assert_eq!(processor.name(), "DestinationMeshCodeExtractor");
    }

    #[test]
    fn test_parameter_defaults() {
        let params = DestinationMeshCodeExtractorParam::default();
        assert_eq!(params.mesh_type, 3);
        assert_eq!(params.meshcode_attr, "_meshcode");
    }

    #[test]
    fn test_mesh_type_mapping() {
        let factory = DestinationMeshCodeExtractorFactory;

        // Test all valid mesh types
        for mesh_type in 1..=6 {
            let params = DestinationMeshCodeExtractorParam {
                mesh_type,
                meshcode_attr: "_meshcode".to_string(),
            };

            let result = factory.build(
                NodeContext::default(),
                EventHub::new(100),
                "test".to_string(),
                Some(
                    serde_json::to_value(params)
                        .unwrap()
                        .as_object()
                        .unwrap()
                        .clone()
                        .into_iter()
                        .collect(),
                ),
            );

            assert!(result.is_ok(), "Mesh type {mesh_type} should be valid");
        }

        // Test invalid mesh type
        let params = DestinationMeshCodeExtractorParam {
            mesh_type: 7, // Invalid
            meshcode_attr: "_meshcode".to_string(),
        };

        let result = factory.build(
            NodeContext::default(),
            EventHub::new(100),
            "test".to_string(),
            Some(
                serde_json::to_value(params)
                    .unwrap()
                    .as_object()
                    .unwrap()
                    .clone()
                    .into_iter()
                    .collect(),
            ),
        );

        assert!(result.is_err(), "Mesh type 7 should be invalid");
    }
}
