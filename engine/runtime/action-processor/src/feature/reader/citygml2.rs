use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{Attributes, Code, CompiledCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::citygml_parser::parser::{CityGmlVersion, Parser};
use crate::citygml_parser::pipeline::build_features;
use crate::feature::errors::FeatureProcessorError;

#[derive(Debug, Clone, Default)]
pub(crate) struct FeatureCityGml2ReaderFactory;

impl ProcessorFactory for FeatureCityGml2ReaderFactory {
    fn name(&self) -> &str {
        "Feature CityGML 2 Reader"
    }

    fn description(&self) -> &str {
        "Reads CityGML 2.0 files, resolving gml:id references and xlink:href links across files."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureCityGml2ReaderParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Input"]
    }

    fn tags(&self) -> &[&'static str] {
        &["citygml", "3d"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureCityGml2ReaderParam = if let Some(ref with) = with {
            let value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::FileCityGml2ReaderFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::FileCityGml2ReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::FileCityGml2ReaderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let dataset = params
            .dataset
            .compile()
            .map_err(|e| FeatureProcessorError::FileCityGml2ReaderFactory(format!("{e:?}")))?;

        let extract_tags: HashSet<String> = params.extract_tags.into_iter().collect();
        let parser = Parser::with_owner_tracking(true, CityGmlVersion::V2, extract_tags.clone());

        Ok(Box::new(FeatureCityGml2Reader {
            dataset,
            extract_tags,
            keep_attributes: params.keep_attributes,
            flatten_single_child_objects: params.flatten_single_child_objects,
            flatten_measure_types: params.flatten_measure_types,
            city_gml_attributes_key: params.city_gml_attributes_key,
            inherit_input_attributes: params.inherit_input_attributes,
            parser,
            base_attributes: HashMap::new(),
        }))
    }
}

/// # Feature CityGML 2 Reader Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCityGml2ReaderParam {
    /// # Dataset
    /// Path expression resolving to the CityGML 2.0 file to read.
    dataset: Code,
    /// # Extract Tags
    /// Feature type names to flatten as individual features. Accepts qualified (`bldg:Building`),
    /// local (`Building`), or Clark notation (`{http://…}Building`). Empty means emit all
    /// top-level city objects unchanged.
    #[serde(default)]
    extract_tags: Vec<String>,
    /// # Keep Attributes
    /// When false, XML attributes (`@`-prefixed entries such as `@gml:id`, `@codeSpace`) are
    /// dropped from parsed features. Defaults to true.
    #[serde(default = "default_keep_attributes")]
    keep_attributes: bool,
    /// # Flatten Single-Child Object Nodes
    /// When true, a wrapper element whose only content is a single child element is dropped: the
    /// child is hoisted up and keyed by its own tag name, always wrapped in an array. Defaults to
    /// false.
    #[serde(default)]
    flatten_single_child_objects: bool,
    /// # Flatten Measure Types
    /// When true, elements with a single `uom` attribute and numeric text content are converted to
    /// a number value, with the unit stored as a sibling `{name}_uom` key. Defaults to false.
    #[serde(default)]
    flatten_measure_types: bool,
    /// # City GML Attributes Key
    /// When set, parsed CityGML attributes are nested under this key in the output feature.
    /// When null, attributes are emitted at the top level. Defaults to null.
    #[serde(default)]
    city_gml_attributes_key: Option<String>,
    /// # Inherit Input Attributes
    /// When true, the input feature's attributes are merged into every feature parsed from its
    /// file. Defaults to true.
    #[serde(default = "default_inherit_input_attributes")]
    inherit_input_attributes: bool,
}

fn default_keep_attributes() -> bool {
    true
}

fn default_inherit_input_attributes() -> bool {
    true
}

pub struct FeatureCityGml2Reader {
    dataset: CompiledCode,
    extract_tags: HashSet<String>,
    keep_attributes: bool,
    flatten_single_child_objects: bool,
    flatten_measure_types: bool,
    city_gml_attributes_key: Option<String>,
    inherit_input_attributes: bool,
    parser: Parser,
    /// Input feature attributes keyed by resolved source file URL, merged into parsed features
    /// when `inherit_input_attributes` is set.
    base_attributes: HashMap<String, Attributes>,
}

impl std::fmt::Debug for FeatureCityGml2Reader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeatureCityGml2Reader")
            .field("parser", &self.parser)
            .finish_non_exhaustive()
    }
}

impl Clone for FeatureCityGml2Reader {
    fn clone(&self) -> Self {
        Self {
            dataset: self.dataset.clone(),
            extract_tags: self.extract_tags.clone(),
            keep_attributes: self.keep_attributes,
            flatten_single_child_objects: self.flatten_single_child_objects,
            flatten_measure_types: self.flatten_measure_types,
            city_gml_attributes_key: self.city_gml_attributes_key.clone(),
            inherit_input_attributes: self.inherit_input_attributes,
            parser: Parser::with_owner_tracking(
                true,
                CityGmlVersion::V2,
                self.extract_tags.clone(),
            ),
            base_attributes: HashMap::new(),
        }
    }
}

impl Processor for FeatureCityGml2Reader {
    fn num_threads(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let path = self
            .dataset
            .eval_string(&ctx.feature, ctx.variables.clone())
            .map_err(|e| {
                FeatureProcessorError::FileCityGml2Reader(format!("Failed to eval dataset: {e:?}"))
            })?;

        let uri = Uri::from_str(&path).map_err(|e| {
            FeatureProcessorError::FileCityGml2Reader(format!("Invalid URI `{path}`: {e}"))
        })?;
        let source_url: Url = uri.clone().into();
        if self.inherit_input_attributes {
            self.base_attributes.insert(
                source_url.as_str().to_string(),
                (*ctx.feature.attributes).clone(),
            );
        }

        let storage = ctx.storage_resolver.resolve(&uri).map_err(|e| {
            FeatureProcessorError::FileCityGml2Reader(format!("Storage resolve error: {e}"))
        })?;
        let bytes = storage.get_sync(uri.path().as_path()).map_err(|e| {
            FeatureProcessorError::FileCityGml2Reader(format!("File read error: {e}"))
        })?;

        self.parser
            .parse(&bytes, &source_url)
            .map_err(|e| FeatureProcessorError::FileCityGml2Reader(format!("{e}")))?;
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        // This reader's own param stays a simple bool; the shared pipeline
        // function takes a caller-declared attribute-name list, so translate
        // at the boundary rather than changing this reader's exposed shape.
        let flatten_leaf_attributes: Vec<String> = if self.flatten_measure_types {
            vec!["uom".to_string()]
        } else {
            Vec::new()
        };
        let next_parser =
            Parser::with_owner_tracking(true, CityGmlVersion::V2, self.extract_tags.clone());
        for feature in build_features(
            std::mem::replace(&mut self.parser, next_parser),
            &self.extract_tags,
            &self.base_attributes,
            self.city_gml_attributes_key.as_deref(),
            self.keep_attributes,
            self.flatten_single_child_objects,
            &flatten_leaf_attributes,
        ) {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature,
                FEATURES_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "Feature CityGML 2 Reader"
    }
}
