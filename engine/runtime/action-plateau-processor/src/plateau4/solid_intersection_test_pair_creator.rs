use super::errors::PlateauProcessorError;
use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};

static PORT_A: Lazy<Port> = Lazy::new(|| Port::new("A"));
static PORT_B: Lazy<Port> = Lazy::new(|| Port::new("B"));

#[derive(Debug, Clone, Default)]
pub struct SolidIntersectionTestPairCreatorFactory;

impl ProcessorFactory for SolidIntersectionTestPairCreatorFactory {
    fn name(&self) -> &str {
        "PLATEAU4.SolidIntersectionTestPairCreator"
    }

    fn description(&self) -> &str {
        "Creates pairs of features from AreaOnAreaOverlayer output for solid intersection testing"
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
                PlateauProcessorError::SolidIntersectionTestPairCreatorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::SolidIntersectionTestPairCreatorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            SolidIntersectionTestPairCreatorParam::default()
        };
        let processor = SolidIntersectionTestPairCreator {
            pair_id_attribute: params.pair_id_attribute,
            list_attribute: params.list_attribute,
            gml_id_attribute: params.gml_id_attribute,
            seen_pairs: BTreeSet::new(),
            feature_cache: HashMap::new(),
            next_pair_id: 1,
        };
        Ok(Box::new(processor))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SolidIntersectionTestPairCreatorParam {
    /// Attribute name to store the pair ID (default: "pair_id")
    #[serde(default = "default_pair_id_attribute")]
    pair_id_attribute: String,

    /// Attribute name containing the list of overlapping features from AreaOnAreaOverlayer (default: "list")
    #[serde(default = "default_list_attribute")]
    list_attribute: String,

    /// Attribute name for the GML ID within the list items (default: "gmlId")
    #[serde(default = "default_gml_id_attribute")]
    gml_id_attribute: String,
}

impl Default for SolidIntersectionTestPairCreatorParam {
    fn default() -> Self {
        Self {
            pair_id_attribute: default_pair_id_attribute(),
            list_attribute: default_list_attribute(),
            gml_id_attribute: default_gml_id_attribute(),
        }
    }
}

fn default_pair_id_attribute() -> String {
    "pair_id".to_string()
}

fn default_list_attribute() -> String {
    "list".to_string()
}

fn default_gml_id_attribute() -> String {
    "gmlId".to_string()
}

/// Represents a pair of GML IDs in canonical order (smaller ID first)
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct GmlIdPair(String, String);

impl GmlIdPair {
    fn new(id1: String, id2: String) -> Self {
        if id1 <= id2 {
            Self(id1, id2)
        } else {
            Self(id2, id1)
        }
    }
}

#[derive(Debug, Clone)]
pub struct SolidIntersectionTestPairCreator {
    pair_id_attribute: String,
    list_attribute: String,
    gml_id_attribute: String,
    /// Set of seen GML ID pairs to avoid duplicates
    seen_pairs: BTreeSet<GmlIdPair>,
    /// Cache of features by GML ID for lookup when creating pairs
    feature_cache: HashMap<String, Feature>,
    next_pair_id: u64,
}

impl Processor for SolidIntersectionTestPairCreator {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        // Get the list attribute from the AreaOnAreaOverlayer output
        let list_attr = feature
            .attributes
            .get(&Attribute::new(&self.list_attribute))
            .ok_or_else(|| {
                PlateauProcessorError::SolidIntersectionTestPairCreator(format!(
                    "Missing '{}' attribute. This processor expects input from AreaOnAreaOverlayer with generateList configured.",
                    self.list_attribute
                ))
            })?;

        let list = match list_attr {
            AttributeValue::Array(arr) => arr,
            _ => {
                return Err(
                    PlateauProcessorError::SolidIntersectionTestPairCreator(format!(
                        "'{}' attribute must be an array",
                        self.list_attribute
                    ))
                    .into(),
                );
            }
        };

        // Extract GML IDs from the list
        let gml_ids: Vec<String> = list
            .iter()
            .filter_map(|item| {
                if let AttributeValue::Map(map) = item {
                    map.get(&self.gml_id_attribute).and_then(|v| {
                        if let AttributeValue::String(s) = v {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            })
            .collect();

        // Need at least 2 features to create a pair
        if gml_ids.len() < 2 {
            return Ok(());
        }

        // Cache features for later lookup - extract feature data from the list
        for item in list {
            if let AttributeValue::Map(map) = item {
                if let Some(AttributeValue::String(gml_id)) = map.get(&self.gml_id_attribute) {
                    if !self.feature_cache.contains_key(gml_id) {
                        // Create a feature from the map data
                        let mut cached_feature = Feature::new();
                        for (key, value) in map {
                            cached_feature
                                .attributes
                                .insert(Attribute::new(key), value.clone());
                        }
                        // Copy geometry from the original overlay feature (it's the intersection area)
                        // Note: For solid intersection test, we need the original solid geometries,
                        // which should be stored in the list item attributes or retrieved separately
                        self.feature_cache.insert(gml_id.clone(), cached_feature);
                    }
                }
            }
        }

        // Create pairs for all combinations
        for i in 0..gml_ids.len() {
            for j in (i + 1)..gml_ids.len() {
                let pair = GmlIdPair::new(gml_ids[i].clone(), gml_ids[j].clone());

                // Skip if we've already seen this pair
                if self.seen_pairs.contains(&pair) {
                    continue;
                }

                // Mark this pair as seen
                self.seen_pairs.insert(pair.clone());

                // Get features for the pair
                let feature_a = self.feature_cache.get(&pair.0);
                let feature_b = self.feature_cache.get(&pair.1);

                if let (Some(feat_a), Some(feat_b)) = (feature_a, feature_b) {
                    let pair_id =
                        AttributeValue::Number(serde_json::Number::from(self.next_pair_id));
                    self.next_pair_id += 1;

                    // Clone features and add pair_id attribute
                    let mut output_a = feat_a.clone();
                    let mut output_b = feat_b.clone();

                    output_a
                        .attributes
                        .insert(Attribute::new(&self.pair_id_attribute), pair_id.clone());
                    output_b
                        .attributes
                        .insert(Attribute::new(&self.pair_id_attribute), pair_id);

                    // Send features to respective ports
                    fw.send(ctx.new_with_feature_and_port(output_a, PORT_A.clone()));
                    fw.send(ctx.new_with_feature_and_port(output_b, PORT_B.clone()));
                }
            }
        }

        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "SolidIntersectionTestPairCreator"
    }
}
