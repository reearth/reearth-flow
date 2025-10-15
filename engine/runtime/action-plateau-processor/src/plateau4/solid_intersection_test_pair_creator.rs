use super::errors::PlateauProcessorError;
use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, Geometry};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

static PORT_A: Lazy<Port> = Lazy::new(|| Port::new("A"));
static PORT_B: Lazy<Port> = Lazy::new(|| Port::new("B"));

#[derive(Debug, Clone, Default)]
pub struct SolidIntersectionTestPairCreatorFactory;

impl ProcessorFactory for SolidIntersectionTestPairCreatorFactory {
    fn name(&self) -> &str {
        "PLATEAU4.SolidIntersectionTestPairCreator"
    }

    fn description(&self) -> &str {
        "Creates pairs of features that can possibly intersect based on bounding box overlap"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(SolidIntersectionTestPairCreatorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![reearth_flow_runtime::node::DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![PORT_A.clone(), PORT_B.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: SolidIntersectionTestPairCreatorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::SolidIntersectionTestPairCreator(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::SolidIntersectionTestPairCreator(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            SolidIntersectionTestPairCreatorParam::default()
        };
        let processor = SolidIntersectionTestPairCreator {
            pair_id_attribute: params.pair_id_attribute,
            bounding_box_attribute: params.bounding_box_attribute,
            features: Vec::new(),
            next_pair_id: 1,
        };
        Ok(Box::new(processor))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SolidIntersectionTestPairCreatorParam {
    #[serde(default = "default_pair_id_attribute")]
    pair_id_attribute: String,
    #[serde(default = "default_bounding_box_attribute")]
    bounding_box_attribute: String,
}

impl Default for SolidIntersectionTestPairCreatorParam {
    fn default() -> Self {
        Self {
            pair_id_attribute: default_pair_id_attribute(),
            bounding_box_attribute: default_bounding_box_attribute(),
        }
    }
}

fn default_pair_id_attribute() -> String {
    "pair_id".to_string()
}

fn default_bounding_box_attribute() -> String {
    "bounding_box".to_string()
}

#[derive(Debug, Clone)]
struct FeatureWithBBox {
    feature: Feature,
    bbox: BBox,
}

#[derive(Debug, Clone)]
struct BBox {
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
    min_z: f64,
    max_z: f64,
}

impl BBox {
    fn overlaps(&self, other: &BBox) -> bool {
        !(self.max_x < other.min_x
            || self.min_x > other.max_x
            || self.max_y < other.min_y
            || self.min_y > other.max_y
            || self.max_z < other.min_z
            || self.min_z > other.max_z)
    }
}

#[derive(Debug, Clone)]
pub struct SolidIntersectionTestPairCreator {
    pair_id_attribute: String,
    bounding_box_attribute: String,
    features: Vec<FeatureWithBBox>,
    next_pair_id: u64,
}

impl Processor for SolidIntersectionTestPairCreator {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();

        // Calculate bounding box if not present
        let bbox = if let Some(bbox_attr) = feature
            .attributes
            .get(&Attribute::new(&self.bounding_box_attribute))
        {
            // Parse existing bounding box from attribute
            if let AttributeValue::Map(map) = bbox_attr {
                BBox {
                    min_x: extract_number_from_string_map(map, "min_x")?,
                    max_x: extract_number_from_string_map(map, "max_x")?,
                    min_y: extract_number_from_string_map(map, "min_y")?,
                    max_y: extract_number_from_string_map(map, "max_y")?,
                    min_z: extract_number_from_string_map(map, "min_z")?,
                    max_z: extract_number_from_string_map(map, "max_z")?,
                }
            } else {
                calculate_bounding_box(&feature.geometry)?
            }
        } else {
            let bbox = calculate_bounding_box(&feature.geometry)?;

            // Add bounding box as an attribute
            let mut bbox_map = HashMap::new();
            bbox_map.insert(
                "min_x".to_string(),
                AttributeValue::Number(serde_json::Number::from_f64(bbox.min_x).unwrap()),
            );
            bbox_map.insert(
                "max_x".to_string(),
                AttributeValue::Number(serde_json::Number::from_f64(bbox.max_x).unwrap()),
            );
            bbox_map.insert(
                "min_y".to_string(),
                AttributeValue::Number(serde_json::Number::from_f64(bbox.min_y).unwrap()),
            );
            bbox_map.insert(
                "max_y".to_string(),
                AttributeValue::Number(serde_json::Number::from_f64(bbox.max_y).unwrap()),
            );
            bbox_map.insert(
                "min_z".to_string(),
                AttributeValue::Number(serde_json::Number::from_f64(bbox.min_z).unwrap()),
            );
            bbox_map.insert(
                "max_z".to_string(),
                AttributeValue::Number(serde_json::Number::from_f64(bbox.max_z).unwrap()),
            );

            feature.attributes.insert(
                Attribute::new(&self.bounding_box_attribute),
                AttributeValue::Map(bbox_map),
            );

            bbox
        };

        // Store the feature with its bounding box
        self.features.push(FeatureWithBBox { feature, bbox });

        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        let mut next_pair_id = self.next_pair_id;

        // Create pairs for all features that have overlapping bounding boxes
        for i in 0..self.features.len() {
            for j in (i + 1)..self.features.len() {
                if self.features[i].bbox.overlaps(&self.features[j].bbox) {
                    // Create a pair
                    let pair_id = AttributeValue::Number(serde_json::Number::from(next_pair_id));
                    next_pair_id += 1;

                    // Clone features and add pair_id attribute
                    let mut feature_a = self.features[i].feature.clone();
                    let mut feature_b = self.features[j].feature.clone();

                    feature_a
                        .attributes
                        .insert(Attribute::new(&self.pair_id_attribute), pair_id.clone());
                    feature_b
                        .attributes
                        .insert(Attribute::new(&self.pair_id_attribute), pair_id);

                    // Send features to respective ports
                    fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                        &ctx,
                        feature_a,
                        PORT_A.clone(),
                    ));
                    fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                        &ctx,
                        feature_b,
                        PORT_B.clone(),
                    ));
                }
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "SolidIntersectionTestPairCreator"
    }
}

fn extract_number_from_string_map(
    map: &HashMap<String, AttributeValue>,
    key: &str,
) -> Result<f64, BoxedError> {
    map.get(key)
        .and_then(|v| {
            if let AttributeValue::Number(n) = v {
                n.as_f64()
            } else {
                None
            }
        })
        .ok_or_else(|| {
            PlateauProcessorError::SolidIntersectionTestPairCreator(format!(
                "Failed to extract {key} from bounding box"
            ))
            .into()
        })
}

fn calculate_bounding_box(geometry: &Geometry) -> Result<BBox, BoxedError> {
    use reearth_flow_geometry::algorithm::bounding_rect::BoundingRect;
    use reearth_flow_geometry::types::no_value::NoValue;
    use reearth_flow_geometry::types::rect::Rect;
    use reearth_flow_types::GeometryValue;

    match &geometry.value {
        GeometryValue::FlowGeometry2D(geom) => {
            let rect_opt: Option<Rect<f64, NoValue>> = geom.bounding_rect();
            if let Some(rect) = rect_opt {
                Ok(BBox {
                    min_x: rect.min().x,
                    max_x: rect.max().x,
                    min_y: rect.min().y,
                    max_y: rect.max().y,
                    min_z: 0.0,
                    max_z: 0.0,
                })
            } else {
                Err(PlateauProcessorError::SolidIntersectionTestPairCreator(
                    "Failed to calculate 2D bounding box".to_string(),
                )
                .into())
            }
        }
        GeometryValue::FlowGeometry3D(geom) => {
            let rect_opt: Option<Rect<f64, f64>> = geom.bounding_rect();
            if let Some(rect) = rect_opt {
                Ok(BBox {
                    min_x: rect.min().x,
                    max_x: rect.max().x,
                    min_y: rect.min().y,
                    max_y: rect.max().y,
                    min_z: rect.min().z,
                    max_z: rect.max().z,
                })
            } else {
                Err(PlateauProcessorError::SolidIntersectionTestPairCreator(
                    "Failed to calculate 3D bounding box".to_string(),
                )
                .into())
            }
        }
        GeometryValue::CityGmlGeometry(cg) => {
            let max_min = cg.max_min_vertice();
            Ok(BBox {
                min_x: max_min.min_lng,
                max_x: max_min.max_lng,
                min_y: max_min.min_lat,
                max_y: max_min.max_lat,
                min_z: max_min.min_height,
                max_z: max_min.max_height,
            })
        }
        GeometryValue::None => Err(PlateauProcessorError::SolidIntersectionTestPairCreator(
            "No geometry present".to_string(),
        )
        .into()),
    }
}
