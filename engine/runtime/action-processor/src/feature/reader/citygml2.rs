use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use reearth_flow_common::uri::Uri;
use reearth_flow_diagnostics::{DiagnosticDraft, ErrorCode};
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

use crate::feature::errors::FeatureProcessorError;
use reearth_flow_citygml::parser::{CityGmlVersion, Parser};
use reearth_flow_citygml::pipeline::build_features;

#[derive(Debug, Clone, Default)]
pub(crate) struct FeatureCityGml2ReaderFactory;

impl ProcessorFactory for FeatureCityGml2ReaderFactory {
    fn name(&self) -> &str {
        "Feature CityGML 2 Reader"
    }

    fn description(&self) -> &str {
        "Reads the CityGML 2.0 file each incoming feature points at, resolving gml:id and \
         xlink:href references across every file read. The attributes of the feature naming a \
         file are carried onto the features parsed from it."
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

        let dataset = params.dataset.compile().map_err(|e| {
            FeatureProcessorError::FileCityGml2ReaderFactory(format!(
                "Failed to compile the dataset expression: {e}"
            ))
        })?;

        let extract_tags: HashSet<String> = params.extract_tags.into_iter().collect();
        let parser = Parser::with_extract_tags(CityGmlVersion::V2, extract_tags.clone());

        Ok(Box::new(FeatureCityGml2Reader {
            dataset,
            extract_tags,
            keep_attributes: params.keep_attributes,
            flatten_single_child_objects: params.flatten_single_child_objects,
            flatten_measure_types: params.flatten_measure_types,
            city_gml_attributes_key: params.city_gml_attributes_key,
            keep_code_space: params.keep_code_space,
            inherit_input_attributes: params.inherit_input_attributes,
            parser,
            base_attributes: HashMap::new(),
            parse_failed: false,
        }))
    }
}

/// # Feature CityGML 2 Reader Parameters
///
/// Which file to read, and how its elements become feature attributes.
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
    /// # CityGML Attributes Key
    /// When set, parsed CityGML attributes are nested under this key in the output feature.
    /// When null, attributes are emitted at the top level. Defaults to null.
    #[serde(default)]
    city_gml_attributes_key: Option<String>,
    /// # Inherit Input Attributes
    /// When true, the input feature's attributes are merged into every feature parsed from its
    /// file. Defaults to true.
    #[serde(default = "default_inherit_input_attributes")]
    inherit_input_attributes: bool,
    /// # Keep Code Space
    /// When true, a coded value resolved against its codelist also keeps the codelist path, as
    /// the document wrote it, in a sibling `{name}_codeSpace` key. Defaults to false.
    #[serde(default)]
    keep_code_space: bool,
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
    keep_code_space: bool,
    inherit_input_attributes: bool,
    parser: Parser,
    /// Input feature attributes keyed by resolved source file URL, merged into parsed features
    /// when `inherit_input_attributes` is set.
    base_attributes: HashMap<String, Attributes>,
    /// Set when a file fails to parse. `Parser::parse` streams, so by then the
    /// parser already holds the part of that file read before the break, and
    /// `finish` must not emit it.
    parse_failed: bool,
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
            keep_code_space: self.keep_code_space,
            inherit_input_attributes: self.inherit_input_attributes,
            parser: Parser::with_extract_tags(CityGmlVersion::V2, self.extract_tags.clone()),
            base_attributes: HashMap::new(),
            parse_failed: false,
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
                FeatureProcessorError::FileCityGml2Reader(format!(
                    "Failed to evaluate the dataset expression: {e}"
                ))
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

        if let Err(e) = self.parser.parse(&bytes, &source_url) {
            // Classify before failing (Action Standard §9). At its default of
            // fatal, `report` records `citygml.parse_failed` in the node's fatal
            // slot, which keeps the first fatal, so it wins over the generic
            // `internal.unclassified` the runtime adds for the `Err` below.
            //
            // The read fails whatever the policy says. `Parser::parse` streams,
            // committing each city object as it reads it, so a file that breaks
            // partway has already added its first objects to the parser, and
            // `finish` would emit them as though the file were complete.
            // Honouring a relaxed disposition would turn a broken file into
            // silently partial output (§4.3, §9 on state left behind), so a
            // relaxed policy is refused with the reason instead.
            self.parse_failed = true;
            let relaxed = ctx
                .report(DiagnosticDraft::new(ErrorCode::CitygmlParseFailed))
                .is_ok();
            let refused = if relaxed {
                " (citygml.parse_failed cannot be relaxed at this reader: part of the \
                 file may already have been read, and skipping it would emit that part \
                 as though it were complete)"
            } else {
                ""
            };
            return Err(FeatureProcessorError::FileCityGml2Reader(format!(
                "{source_url}: {e}{refused}"
            ))
            .into());
        }
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        if self.parse_failed {
            // The runtime still calls `finish` after a fatal in `process`, so
            // without this the part of the broken file read before the break
            // would go downstream inside a run already reported as failed.
            // Emitting nothing matches the source readers, which send zero
            // features from a document they could not read.
            self.parser = Parser::with_extract_tags(CityGmlVersion::V2, self.extract_tags.clone());
            return Ok(());
        }
        // This reader's own param stays a simple bool; the shared pipeline
        // function takes a caller-declared attribute-name list, so translate
        // at the boundary rather than changing this reader's exposed shape.
        let flatten_leaf_attributes: Vec<String> = if self.flatten_measure_types {
            vec!["uom".to_string()]
        } else {
            Vec::new()
        };
        let next_parser = Parser::with_extract_tags(CityGmlVersion::V2, self.extract_tags.clone());
        for feature in build_features(
            std::mem::replace(&mut self.parser, next_parser),
            &self.extract_tags,
            &self.base_attributes,
            self.city_gml_attributes_key.as_deref(),
            self.keep_attributes,
            self.flatten_single_child_objects,
            &flatten_leaf_attributes,
            self.keep_code_space,
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

/// Action Standard §9 on the one fallible path that has a CityGML cause. These
/// run in both geometry worlds: both parsers stream, so both have the property
/// the refusal below depends on.
#[cfg(test)]
mod parse_failure_tests {
    use std::sync::Arc;

    use reearth_flow_diagnostics::{Disposition, DispositionPolicy, OverrideInput, PolicyInput};
    use reearth_flow_runtime::diagnostics::NodeDiagnosticsHandle;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_runtime::node::NodeHandle;
    use reearth_flow_types::Feature;

    use super::*;

    /// The first city object is complete; the second closes an element with
    /// the wrong tag, so the XML breaks partway through the document.
    const BREAKS_PARTWAY: &str = concat!(
        r#"<?xml version="1.0" encoding="UTF-8"?>"#,
        r#"<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0""#,
        r#" xmlns:bldg="http://www.opengis.net/citygml/building/2.0""#,
        r#" xmlns:gml="http://www.opengis.net/gml">"#,
        r#"<core:cityObjectMember><bldg:Building gml:id="b1">"#,
        r#"<bldg:function>1000</bldg:function>"#,
        r#"</bldg:Building></core:cityObjectMember>"#,
        r#"<core:cityObjectMember><bldg:Building gml:id="b2">"#,
        r#"<bldg:function>1000</bldg:usage>"#,
        r#"</bldg:Building></core:cityObjectMember>"#,
        r#"</core:CityModel>"#,
    );

    fn handle(policy: DispositionPolicy) -> Arc<NodeDiagnosticsHandle> {
        Arc::new(NodeDiagnosticsHandle::new(
            "n1".to_string(),
            NodeHandle::for_test("n1"),
            "processor".into(),
            "Feature CityGML 2 Reader".into(),
            Arc::default(),
            Arc::new(policy),
            false,
        ))
    }

    fn relaxing_parse_failed() -> DispositionPolicy {
        DispositionPolicy::compile(PolicyInput {
            overrides: vec![OverrideInput {
                node: None,
                code: Some("citygml.parse_failed".to_string()),
                category: None,
                disposition: Disposition::WarnDrop,
            }],
            ..Default::default()
        })
        .expect("a code-only override should compile")
    }

    fn reader(url: &str) -> FeatureCityGml2Reader {
        FeatureCityGml2Reader {
            dataset: CompiledCode::Literal(url.to_string()),
            extract_tags: HashSet::new(),
            keep_attributes: true,
            flatten_single_child_objects: false,
            flatten_measure_types: false,
            city_gml_attributes_key: None,
            keep_code_space: false,
            inherit_input_attributes: true,
            parser: Parser::with_extract_tags(CityGmlVersion::V2, HashSet::new()),
            base_attributes: HashMap::new(),
            parse_failed: false,
        }
    }

    /// Writes `content` to a file of its own and returns its `file://` URL.
    /// Named per test, because tests in one binary run concurrently.
    fn fixture(name: &str, content: &str) -> (std::path::PathBuf, String) {
        let path = std::env::temp_dir().join(format!(
            "reearth-flow-citygml2-{name}-{}.gml",
            std::process::id()
        ));
        std::fs::write(&path, content).expect("the fixture should be writable");
        let url = format!("file://{}", path.display());
        (path, url)
    }

    /// Runs `process` over the fixture with a real diagnostics handle, so the
    /// policy is actually consulted rather than falling back to the default.
    fn process_with(
        name: &str,
        policy: DispositionPolicy,
    ) -> (
        FeatureCityGml2Reader,
        Result<(), BoxedError>,
        Arc<NodeDiagnosticsHandle>,
    ) {
        let (path, url) = fixture(name, BREAKS_PARTWAY);
        let mut reader = reader(&url);
        let handle = handle(policy);
        let mut ctx = ExecutorContext::new_with_node_context_feature_and_port(
            &NodeContext::default(),
            Feature::new_with_attributes(Attributes::new()),
            FEATURES_PORT.clone(),
        );
        ctx.diagnostics = Some(handle.clone());
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let result = reader.process(ctx, &fw);
        let _ = std::fs::remove_file(path);
        (reader, result, handle)
    }

    #[test]
    fn a_parse_failure_fails_the_read_classified_as_citygml_parse_failed() {
        let (_, result, handle) = process_with("default", DispositionPolicy::default());
        assert!(
            result.is_err(),
            "a document that cannot be parsed must fail the read"
        );
        let fatal = handle
            .inner
            .take_fatal()
            .expect("the classified code must occupy the fatal slot");
        assert_eq!(
            fatal.code,
            ErrorCode::CitygmlParseFailed,
            "the code must win the first-fatal slot over the runtime's generic wrapper"
        );
    }

    #[test]
    fn a_policy_relaxing_the_code_is_refused_rather_than_skipping_the_file() {
        let (_, result, _) = process_with("relaxed", relaxing_parse_failed());
        let err = result.expect_err("relaxing must not turn the failure into a skip");
        let msg = err.to_string();
        assert!(
            msg.contains("cannot be relaxed"),
            "the refusal must say why the policy was not honoured, got: {msg}"
        );
    }

    /// The fix for the premise pinned below: after a parse failure, nothing
    /// from the half-read parser reaches the output.
    #[test]
    fn after_a_parse_failure_finish_emits_nothing_from_the_half_read_file() {
        let (mut reader, result, _) = process_with("premise", DispositionPolicy::default());
        assert!(result.is_err());
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        reader
            .finish(NodeContext::default(), &fw)
            .expect("finish itself should succeed");
        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("built as a noop forwarder");
        };
        let emitted = noop
            .send_ports
            .lock()
            .unwrap()
            .iter()
            .filter(|port| **port == *FEATURES_PORT)
            .count();
        assert_eq!(
            emitted, 0,
            "b1 was committed before the XML broke at b2 and must not be emitted"
        );
    }

    /// The premise behind both the refusal and `finish` emitting nothing:
    /// `Parser::parse` commits each city object as it reads it. If this ever
    /// fails because the parser gained rollback, a relaxed policy could safely
    /// become a skip of just the broken file.
    #[test]
    fn the_parser_commits_objects_read_before_the_xml_breaks() {
        let mut parser = Parser::with_extract_tags(CityGmlVersion::V2, HashSet::new());
        let url = Url::parse("file:///breaks-partway.gml").unwrap();
        assert!(parser.parse(BREAKS_PARTWAY.as_bytes(), &url).is_err());
        let features = reearth_flow_citygml::pipeline::build_features(
            parser,
            &HashSet::new(),
            &HashMap::new(),
            None,
            true,
            false,
            &[],
            false,
        );
        assert_eq!(features.len(), 1, "b1 is already in the parser");
    }
}
