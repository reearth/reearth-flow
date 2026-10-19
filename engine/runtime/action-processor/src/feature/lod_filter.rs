use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{Context, ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{lod::LodMask, Attribute, AttributeValue, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;

static UP_TO_LOD0: Lazy<Port> = Lazy::new(|| Port::new("up_to_lod0"));
static UP_TO_LOD1: Lazy<Port> = Lazy::new(|| Port::new("up_to_lod1"));
static UP_TO_LOD2: Lazy<Port> = Lazy::new(|| Port::new("up_to_lod2"));
static UP_TO_LOD3: Lazy<Port> = Lazy::new(|| Port::new("up_to_lod3"));
static UP_TO_LOD4: Lazy<Port> = Lazy::new(|| Port::new("up_to_lod4"));
static UNFILTERED_PORT: Lazy<Port> = Lazy::new(|| Port::new("unfiltered"));

#[derive(Debug, Clone, Default)]
pub(super) struct FeatureLodFilterFactory;

impl ProcessorFactory for FeatureLodFilterFactory {
    fn name(&self) -> &str {
        "FeatureLodFilter"
    }

    fn description(&self) -> &str {
        "Filters features by Level of Detail (LOD), routing them to appropriate output ports"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureLodFilterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            UP_TO_LOD0.clone(),
            UP_TO_LOD1.clone(),
            UP_TO_LOD2.clone(),
            UP_TO_LOD3.clone(),
            UP_TO_LOD4.clone(),
            UNFILTERED_PORT.clone(),
        ]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureLodFilterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::LodFilterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::LodFilterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::LodFilterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let process = FeatureLodFilter {
            filter_key: params.filter_key,
            fallback: params.fallback,
            buffer_features: HashMap::new(),
            max_lod: HashMap::new(),
        };
        Ok(Box::new(process))
    }
}

/// # FeatureLodFilter Parameters
///
/// Configuration for filtering features based on Level of Detail (LOD).
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct FeatureLodFilterParam {
    /// Attribute used to group features for LOD filtering
    filter_key: Attribute,
    /// When true, a feature lacking LOD N is still sent to `up_to_lodN` with its highest
    /// lower LOD, if any feature in its group has LOD N. When false, it is sent only to
    /// the ports of the LODs it has.
    #[serde(default = "default_fallback")]
    fallback: bool,
}

fn default_fallback() -> bool {
    true
}

struct LodCount {
    max_lod: u8,
}

#[derive(Debug, Clone)]
struct FeatureLodFilter {
    filter_key: Attribute,
    fallback: bool,
    buffer_features: HashMap<AttributeValue, Vec<Feature>>,
    max_lod: HashMap<AttributeValue, u8>,
}

impl Processor for FeatureLodFilter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let Some(lod) = feature.metadata.lod else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone()));
            return Ok(());
        };
        let filter_key = feature.get(&self.filter_key).ok_or_else(|| {
            FeatureProcessorError::LodFilter(format!(
                "Failed to get filter key: {}",
                self.filter_key
            ))
        })?;
        if !self.buffer_features.contains_key(filter_key) {
            self.flush_buffer(ctx.as_context(), fw);
            self.buffer_features.clear();
        }
        let features = self.buffer_features.entry(filter_key.clone()).or_default();
        features.push(feature.clone());
        if let Some(highest_lod) = lod.highest_lod() {
            let max_lod = self.max_lod.entry(filter_key.clone()).or_insert(0);
            if highest_lod > *max_lod {
                *max_lod = highest_lod;
            }
        }
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        self.flush_buffer(ctx.as_context(), fw);
        Ok(())
    }

    fn name(&self) -> &str {
        "FeatureLodFilter"
    }
}

impl FeatureLodFilter {
    fn flush_buffer(&self, ctx: Context, fw: &ProcessorChannelForwarder) {
        for (key, features) in self.buffer_features.iter() {
            let lod_count = LodCount {
                max_lod: self.max_lod.get(key).cloned().unwrap_or(0),
            };
            features.iter().for_each(|feature| {
                Self::routing_feature_by_lod(ctx.clone(), fw, feature, &lod_count, self.fallback);
            });
        }
    }

    fn routing_feature_by_lod(
        ctx: Context,
        fw: &ProcessorChannelForwarder,
        feature: &Feature,
        lod_count: &LodCount,
        fallback: bool,
    ) {
        let Some(lod) = feature.metadata.lod else {
            fw.send(ctx.as_executor_context(feature.clone(), UNFILTERED_PORT.clone()));
            return;
        };
        if lod.has_lod(0) {
            let feature = Self::feature_with_single_lod(feature, 0);
            fw.send(ctx.as_executor_context(feature, UP_TO_LOD0.clone()));
        }
        if lod.has_lod(1) {
            let feature = Self::feature_with_single_lod(feature, 1);
            fw.send(ctx.as_executor_context(feature, UP_TO_LOD1.clone()));
        }
        if lod_count.max_lod >= 2
            && (lod.has_lod(2)
                || fallback
                    && ((lod.has_lod(1) && !lod.has_lod(2))
                        || (lod.has_lod(0) && !lod.has_lod(2) && !lod.has_lod(1))))
        {
            let feature = Self::feature_with_single_lod(feature, 2);
            fw.send(ctx.as_executor_context(feature, UP_TO_LOD2.clone()));
        }
        if lod_count.max_lod >= 3
            && (lod.has_lod(3)
                || fallback
                    && ((lod.has_lod(2) && !lod.has_lod(3))
                        || (lod.has_lod(1) && !lod.has_lod(3) && !lod.has_lod(2))))
        {
            let feature = Self::feature_with_single_lod(feature, 3);
            fw.send(ctx.as_executor_context(feature, UP_TO_LOD3.clone()));
        }
        if lod_count.max_lod >= 4
            && (lod.has_lod(4)
                || fallback
                    && ((lod.has_lod(3) && !lod.has_lod(4))
                        || (lod.has_lod(2) && !lod.has_lod(4) && !lod.has_lod(3))
                        || (lod.has_lod(1)
                            && !lod.has_lod(4)
                            && !lod.has_lod(3)
                            && !lod.has_lod(2))))
        {
            let feature = Self::feature_with_single_lod(feature, 4);
            fw.send(ctx.as_executor_context(feature, UP_TO_LOD4.clone()));
        }
    }

    fn feature_with_single_lod(feature: &Feature, max_lod: u8) -> Feature {
        let mut filtered_feature = feature.clone();

        // Calculate the actual LOD to use based on feature's available LODs
        let actual_lod = if let Some(lod_mask) = &feature.metadata.lod {
            // Find the maximum LOD that doesn't exceed max_lod
            let mut best_lod = None;
            for lod in 0..=max_lod {
                if lod_mask.has_lod(lod) {
                    best_lod = Some(lod);
                }
            }
            best_lod
        } else {
            None
        };

        if let Some(target_lod) = actual_lod {
            if let Some(citygml_geometry) = filtered_feature.geometry.value.as_citygml_geometry() {
                let filtered_citygml_geometry =
                    citygml_geometry.filter_by_lod(|gml_geom| gml_geom.lod == Some(target_lod));

                filtered_feature.geometry_mut().value =
                    reearth_flow_types::GeometryValue::CityGmlGeometry(filtered_citygml_geometry);
            }

            if filtered_feature.metadata.lod.is_some() {
                let mut new_lod_mask = LodMask::default();
                new_lod_mask.add_lod(target_lod);
                filtered_feature.metadata.lod = Some(new_lod_mask);
            }
        }

        filtered_feature
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;

    use super::*;
    use crate::tests::utils::create_default_execute_context;

    fn make_feature(id: &str, lods: &[u8]) -> Feature {
        let mut attrs: IndexMap<Attribute, AttributeValue> = IndexMap::new();
        attrs.insert(
            Attribute::new("city_code"),
            AttributeValue::String("13102".to_string()),
        );
        attrs.insert(Attribute::new("id"), AttributeValue::String(id.to_string()));
        let mut feature = Feature::from(attrs);
        let mut mask = LodMask::default();
        for lod in lods {
            mask.add_lod(*lod);
        }
        feature.metadata.lod = Some(mask);
        feature
    }

    /// Runs a LoD2-only feature and a LoD2+LoD3 feature of the same city through the
    /// filter and returns the ids sent to `up_to_lod3`.
    fn ids_sent_to_lod3(fallback: bool) -> Vec<String> {
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let mut processor = FeatureLodFilter {
            filter_key: Attribute::new("city_code"),
            fallback,
            buffer_features: HashMap::new(),
            max_lod: HashMap::new(),
        };
        for feature in [
            make_feature("lod2_only", &[2]),
            make_feature("lod3", &[2, 3]),
        ] {
            let ctx = create_default_execute_context(&feature);
            processor.process(ctx, &fw).unwrap();
        }
        processor.finish(NodeContext::default(), &fw).unwrap();

        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!()
        };
        let features = noop.send_features.lock().unwrap();
        let ports = noop.send_ports.lock().unwrap();
        let mut ids: Vec<String> = features
            .iter()
            .zip(ports.iter())
            .filter(|(_, port)| **port == *UP_TO_LOD3)
            .filter_map(|(f, _)| match f.get(&Attribute::new("id")) {
                Some(AttributeValue::String(s)) => Some(s.clone()),
                _ => None,
            })
            .collect();
        ids.sort();
        ids
    }

    #[test]
    fn fallback_sends_lower_lod_to_higher_port() {
        assert_eq!(ids_sent_to_lod3(true), vec!["lod2_only", "lod3"]);
    }

    #[test]
    fn no_fallback_sends_only_existing_lod() {
        assert_eq!(ids_sent_to_lod3(false), vec!["lod3"]);
    }
}
