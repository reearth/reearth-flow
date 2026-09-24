use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT, REJECTED_PORT},
};
use reearth_flow_types::{AttributeValue, Attributes, Code, CompiledCode, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::feature::errors::FeatureProcessorError;
use reearth_flow_citygml::parser::{CityGmlVersion, Parser};
use reearth_flow_citygml::pipeline::build_features_reporting;

#[derive(Debug, Clone, Default)]
pub(crate) struct FeatureCityGml3ReaderFactory;

impl ProcessorFactory for FeatureCityGml3ReaderFactory {
    fn name(&self) -> &str {
        "Feature CityGML 3 Reader"
    }

    fn description(&self) -> &str {
        "Reads the CityGML 3.0 file each incoming feature points at, resolving gml:id and \
         xlink:href references across every file read. The attributes of the feature naming a \
         file are carried onto the features parsed from it. Coordinate content the file writes \
         but that cannot be read as geometry leaves the city object without that geometry, and \
         each such site is reported on the rejected port."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureCityGml3ReaderParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
    }

    fn tags(&self) -> &[&'static str] {
        &["citygml", "3d"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: FeatureCityGml3ReaderParam = if let Some(ref with) = with {
            let value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::FileCityGml3ReaderFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::FileCityGml3ReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::FileCityGml3ReaderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let dataset = params
            .dataset
            .compile()
            .map_err(|e| FeatureProcessorError::FileCityGml3ReaderFactory(format!("{e:?}")))?;

        let extract_tags: HashSet<String> = params.extract_tags.into_iter().collect();
        let parser = Parser::with_extract_tags(CityGmlVersion::V3, extract_tags.clone());

        Ok(Box::new(FeatureCityGml3Reader {
            dataset,
            extract_tags,
            keep_attributes: params.keep_attributes,
            flatten_single_child_objects: params.flatten_single_child_objects,
            flatten_leaf_attributes: params.flatten_leaf_attributes,
            city_gml_attributes_key: params.city_gml_attributes_key,
            inherit_input_attributes: params.inherit_input_attributes,
            parser,
            file_attributes: HashMap::new(),
        }))
    }
}

/// # Feature CityGML 3 Reader Parameters
///
/// Which file to read, and how its elements become feature attributes.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCityGml3ReaderParam {
    /// # Dataset
    /// Path expression resolving to the CityGML 3.0 file to read.
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
    /// # Flatten Leaf Attributes
    /// Attribute names (e.g. `uom`) that mark a leaf for collapsing: an element with exactly one
    /// XML attribute in this list, no child elements, and numeric text content is converted to a
    /// number value, with the attribute's value stored as a sibling `{name}_{attribute}` key.
    /// Empty (the default) disables this.
    #[serde(default)]
    flatten_leaf_attributes: Vec<String>,
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

pub struct FeatureCityGml3Reader {
    dataset: CompiledCode,
    extract_tags: HashSet<String>,
    keep_attributes: bool,
    flatten_single_child_objects: bool,
    flatten_leaf_attributes: Vec<String>,
    city_gml_attributes_key: Option<String>,
    inherit_input_attributes: bool,
    parser: Parser,
    /// The attributes of the input feature that named each source file, keyed by its resolved
    /// URL. Merged into the features parsed from that file when `inherit_input_attributes` is
    /// set, and used either way to say which file a malformed site was found in.
    file_attributes: HashMap<String, Attributes>,
}

impl std::fmt::Debug for FeatureCityGml3Reader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeatureCityGml3Reader")
            .field("parser", &self.parser)
            .finish_non_exhaustive()
    }
}

impl Clone for FeatureCityGml3Reader {
    fn clone(&self) -> Self {
        Self {
            dataset: self.dataset.clone(),
            extract_tags: self.extract_tags.clone(),
            keep_attributes: self.keep_attributes,
            flatten_single_child_objects: self.flatten_single_child_objects,
            flatten_leaf_attributes: self.flatten_leaf_attributes.clone(),
            city_gml_attributes_key: self.city_gml_attributes_key.clone(),
            inherit_input_attributes: self.inherit_input_attributes,
            parser: Parser::with_extract_tags(CityGmlVersion::V3, self.extract_tags.clone()),
            file_attributes: HashMap::new(),
        }
    }
}

impl Processor for FeatureCityGml3Reader {
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
                FeatureProcessorError::FileCityGml3Reader(format!("Failed to eval dataset: {e:?}"))
            })?;

        let uri = Uri::from_str(&path).map_err(|e| {
            FeatureProcessorError::FileCityGml3Reader(format!("Invalid URI `{path}`: {e}"))
        })?;
        let source_url: Url = uri.clone().into();
        self.file_attributes.insert(
            source_url.as_str().to_string(),
            (*ctx.feature.attributes).clone(),
        );

        let storage = ctx.storage_resolver.resolve(&uri).map_err(|e| {
            FeatureProcessorError::FileCityGml3Reader(format!("Storage resolve error: {e}"))
        })?;
        let bytes = storage.get_sync(uri.path().as_path()).map_err(|e| {
            FeatureProcessorError::FileCityGml3Reader(format!("File read error: {e}"))
        })?;

        self.parser
            .parse(&bytes, &source_url)
            .map_err(|e| FeatureProcessorError::FileCityGml3Reader(format!("{e}")))?;
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let next_parser = Parser::with_extract_tags(CityGmlVersion::V3, self.extract_tags.clone());
        let inherited = if self.inherit_input_attributes {
            self.file_attributes.clone()
        } else {
            HashMap::new()
        };
        let (features, malformations) = build_features_reporting(
            std::mem::replace(&mut self.parser, next_parser),
            &self.extract_tags,
            &inherited,
            self.city_gml_attributes_key.as_deref(),
            self.keep_attributes,
            self.flatten_single_child_objects,
            &self.flatten_leaf_attributes,
        );
        for feature in features {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature,
                FEATURES_PORT.clone(),
            ));
        }

        // Every malformed site is reported one by one rather than folded into the
        // feature it belonged to: a city object can hold several, and a check
        // that counts them needs each one.
        for malformation in malformations {
            // A site reached through an `xlink:href` into a file no input
            // feature named has no attributes to carry; the reported file
            // still says where it was.
            let attributes = self
                .file_attributes
                .get(&malformation.file)
                .cloned()
                .unwrap_or_default();
            let mut feature = Feature::new_with_attributes(attributes);
            feature.insert(
                "malformationFile",
                AttributeValue::String(malformation.file),
            );
            feature.insert(
                "malformationLocation",
                AttributeValue::String(malformation.location),
            );
            feature.insert(
                "malformationReason",
                AttributeValue::String(malformation.reason),
            );
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature,
                REJECTED_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "Feature CityGML 3 Reader"
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::Attribute;

    /// A Building whose only surface carries a `gml:posList` with a non-numeric
    /// token: well-formed XML the geometry layer cannot read.
    const BAD_POSLIST: &str = concat!(
        r#"<?xml version="1.0" encoding="UTF-8"?>"#,
        r#"<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0""#,
        r#" xmlns:bldg="http://www.opengis.net/citygml/building/3.0""#,
        r#" xmlns:gml="http://www.opengis.net/gml/3.2">"#,
        r#"<core:cityObjectMember><bldg:Building gml:id="b1">"#,
        r#"<core:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>"#,
        r#"<gml:Polygon gml:id="p1"><gml:exterior><gml:LinearRing>"#,
        r#"<gml:posList>1.0 nope 3.0</gml:posList>"#,
        r#"</gml:LinearRing></gml:exterior></gml:Polygon>"#,
        r#"</gml:surfaceMember></gml:MultiSurface></core:lod2MultiSurface>"#,
        r#"</bldg:Building></core:cityObjectMember>"#,
        r#"</core:CityModel>"#,
    );

    const SOURCE_URL: &str = "file:///udx/bldg/a.gml";

    /// Parse `BAD_POSLIST` into a reader, run `finish`, and return the features
    /// it sent per port.
    fn parse_and_finish() -> Vec<(Port, Feature)> {
        let mut input_attributes = Attributes::new();
        input_attributes.insert(
            Attribute::new("name"),
            AttributeValue::String("a.gml".into()),
        );

        let mut reader = FeatureCityGml3Reader {
            // `finish` never evaluates it; only `process`, which these tests do
            // not call because they feed the parser directly.
            dataset: CompiledCode::Literal(SOURCE_URL.to_string()),
            extract_tags: HashSet::new(),
            keep_attributes: true,
            flatten_single_child_objects: false,
            flatten_leaf_attributes: Vec::new(),
            city_gml_attributes_key: None,
            inherit_input_attributes: true,
            parser: Parser::with_extract_tags(CityGmlVersion::V3, HashSet::new()),
            file_attributes: HashMap::from([(SOURCE_URL.to_string(), input_attributes)]),
        };
        let url = Url::parse(SOURCE_URL).unwrap();
        reader
            .parser
            .parse(BAD_POSLIST.as_bytes(), &url)
            .expect("the XML is well-formed even though the posList content is not");

        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        reader.finish(NodeContext::default(), &fw).unwrap();

        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("the forwarder is the one built above");
        };
        let ports = noop.send_ports.lock().unwrap().clone();
        let features = noop.send_features.lock().unwrap().clone();
        ports.into_iter().zip(features).collect()
    }

    /// The unreadable site arrives on `rejected` alongside the city object,
    /// naming where it was and what was wrong, so a quality check can count it.
    /// The city object itself still arrives, without that geometry.
    #[test]
    fn reporting_adds_one_feature_naming_the_site() {
        let sent = parse_and_finish();
        assert_eq!(sent.len(), 2);
        assert_eq!(sent[0].0, *FEATURES_PORT);

        let (port, reported) = &sent[1];
        assert_eq!(*port, *REJECTED_PORT);
        assert_eq!(
            reported.attributes.get(&Attribute::new("malformationFile")),
            Some(&AttributeValue::String(SOURCE_URL.to_string()))
        );
        assert_eq!(
            reported
                .attributes
                .get(&Attribute::new("malformationLocation")),
            Some(&AttributeValue::String("p1".to_string()))
        );
        let AttributeValue::String(reason) = reported
            .attributes
            .get(&Attribute::new("malformationReason"))
            .expect("the reported site must say what was wrong")
        else {
            panic!("the reason must be a string");
        };
        assert!(
            reason.contains("posList"),
            "the reason must name what could not be read, got: {reason}"
        );
    }

    /// The reported site carries the attributes of the feature that named the
    /// file, so a check can put it on the same result row as the file's other
    /// findings.
    #[test]
    fn a_reported_site_carries_the_input_file_attributes() {
        let sent = parse_and_finish();
        let (_, reported) = &sent[1];
        assert_eq!(
            reported.attributes.get(&Attribute::new("name")),
            Some(&AttributeValue::String("a.gml".to_string()))
        );
    }
}
