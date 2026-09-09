use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use reearth_flow_citygml::parser::{CityGmlVersion, Parser};
use reearth_flow_citygml::pipeline::build_features_reporting;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, Source, SourceFactory, FEATURES_PORT},
};
use reearth_flow_types::Attributes;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::Sender;
use url::Url;

use super::reader::runner::{
    get_content, get_input_path, FileReaderCommonParam, FileReaderCompiledParam,
};
use crate::errors::SourceError;

#[derive(Debug, Clone, Default)]
pub(crate) struct CityGml2ReaderFactory;

impl SourceFactory for CityGml2ReaderFactory {
    fn name(&self) -> &str {
        "CityGML 2 Reader"
    }

    fn description(&self) -> &str {
        "Reads CityGML 2.0 files as 3D city models, resolving `gml:id` references within each document."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CityGml2ReaderParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Input"]
    }

    fn tags(&self) -> &[&'static str] {
        &["citygml", "3d"]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
        _state: Option<Vec<u8>>,
    ) -> Result<Box<dyn Source>, BoxedError> {
        let params: CityGml2ReaderParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SourceError::CityGml2ReaderFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SourceError::CityGml2ReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SourceError::CityGml2ReaderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let common = params.common_property.compile(&ctx).map_err(|e| {
            SourceError::CityGml2ReaderFactory(format!("Failed to compile params: {e:?}"))
        })?;
        Ok(Box::new(CityGml2Reader {
            common,
            property: params.property,
        }))
    }
}

/// # CityGML 2 Reader Parameters
///
/// Configuration for reading a CityGML 2.0 document as 3D city models.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct CityGml2ReaderParam {
    #[serde(flatten)]
    pub(super) common_property: FileReaderCommonParam,
    #[serde(flatten)]
    pub(super) property: CityGml2Property,
}

/// # CityGML 2 Reader Options
///
/// Controls which city objects become features and how their attributes are shaped.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct CityGml2Property {
    /// # Extract Tags
    /// Feature type names to flatten as individual features. Accepts qualified (`bldg:Building`),
    /// local (`Building`), or Clark notation (`{http://…}Building`). Empty means emit all
    /// top-level city objects unchanged.
    #[serde(default)]
    pub(super) extract_tags: Vec<String>,
    /// # Keep Attributes
    /// When false, XML attributes (`@`-prefixed entries such as `@gml:id`, `@codeSpace`) are
    /// dropped from parsed features. Defaults to true.
    #[serde(default = "default_keep_attributes")]
    pub(super) keep_attributes: bool,
    /// # Flatten Measure Types
    /// When true, elements with a single `uom` attribute and numeric text content are converted to
    /// a number value, with the unit stored as a sibling `{name}_uom` key. Defaults to false.
    #[serde(default)]
    pub(super) flatten_measure_types: bool,
    /// # City GML Attributes Key
    /// When set, parsed CityGML attributes are nested under this key in the output feature.
    /// When null, attributes are emitted at the top level. Defaults to null.
    #[serde(default)]
    pub(super) city_gml_attributes_key: Option<String>,
}

fn default_keep_attributes() -> bool {
    true
}

#[derive(Debug, Clone)]
pub(super) struct CityGml2Reader {
    common: FileReaderCompiledParam,
    property: CityGml2Property,
}

#[async_trait::async_trait]
impl Source for CityGml2Reader {
    async fn initialize(&self, _ctx: NodeContext) {}

    fn name(&self) -> &str {
        "CityGML 2 Reader"
    }

    async fn serialize_state(&self) -> Result<Vec<u8>, BoxedError> {
        Ok(vec![])
    }

    async fn start(
        &mut self,
        ctx: NodeContext,
        sender: Sender<(Port, IngestionMessage)>,
    ) -> Result<(), BoxedError> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let input_path = get_input_path(&self.common)?;
        let content = get_content(&self.common, storage_resolver).await?;

        // `inline` content has no path of its own, but the parser keys its
        // per-file CRS and reference tables on a URL, so give it a stable one.
        let source_url: Url = match input_path {
            Some(uri) => uri.into(),
            None => Url::parse("file:///inline.gml").map_err(|e| {
                SourceError::CityGml2Reader(format!("Failed to build inline source URL: {e}"))
            })?,
        };

        let extract_tags: HashSet<String> = self.property.extract_tags.iter().cloned().collect();
        let mut parser = Parser::with_extract_tags(CityGmlVersion::V2, extract_tags.clone());
        parser
            .parse(&content, &source_url)
            .map_err(|e| SourceError::CityGml2Reader(format!("{source_url}: {e}")))?;

        let flatten_leaf_attributes: Vec<String> = if self.property.flatten_measure_types {
            vec!["uom".to_string()]
        } else {
            Vec::new()
        };

        // `flatten_single_child_objects` is passed false rather than exposed: the
        // new-geometry path ignores it, so a parameter for it would do nothing.
        let (features, malformations) = build_features_reporting(
            parser,
            &extract_tags,
            &HashMap::<String, Attributes>::new(),
            self.property.city_gml_attributes_key.as_deref(),
            self.property.keep_attributes,
            false,
            &flatten_leaf_attributes,
        );
        // Per Action Standard §4.3, present-but-malformed input fails the read
        // naming the offending location, rather than emitting a feature with
        // its geometry silently dropped. Checked before the send loop, so a
        // malformed document sends zero features.
        if let Some(first) = malformations.first() {
            return Err(SourceError::CityGml2Reader(format!(
                "{source_url}: malformed input ({} total): {first}",
                malformations.len()
            ))
            .into());
        }
        for feature in features {
            sender
                .send((
                    FEATURES_PORT.clone(),
                    IngestionMessage::OperationEvent { feature },
                ))
                .await
                .map_err(|e| SourceError::CityGml2Reader(format!("Failed to send feature: {e}")))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use reearth_flow_citygml::pipeline::build_features;
    use reearth_flow_runtime::node::SourceFactory;
    use tokio::sync::mpsc;

    #[test]
    fn factory_metadata_matches_the_action_standard() {
        let factory = CityGml2ReaderFactory;
        assert_eq!(factory.name(), "CityGML 2 Reader");
        assert_eq!(factory.categories(), &["Input"]);
        assert_eq!(factory.tags(), &["citygml", "3d"]);
        assert_eq!(factory.get_output_ports().len(), 1);
        assert!(factory.description().ends_with('.'));
        assert!(factory.parameter_schema().is_some());
    }

    const MINIMAL_CITYGML_2: &str = concat!(
        r#"<?xml version="1.0" encoding="UTF-8"?>"#,
        r#"<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0""#,
        r#" xmlns:bldg="http://www.opengis.net/citygml/building/2.0""#,
        r#" xmlns:gml="http://www.opengis.net/gml">"#,
        r#"<core:cityObjectMember><bldg:Building gml:id="b1">"#,
        r#"<bldg:function>1000</bldg:function>"#,
        r#"</bldg:Building></core:cityObjectMember>"#,
        r#"</core:CityModel>"#,
    );

    #[test]
    fn parses_one_city_object_into_one_feature() {
        let mut parser = Parser::new(CityGmlVersion::V2);
        let url = Url::parse("file:///test.gml").unwrap();
        parser
            .parse(MINIMAL_CITYGML_2.as_bytes(), &url)
            .expect("minimal CityGML 2.0 should parse");
        let features = build_features(
            parser,
            &HashSet::new(),
            &HashMap::<String, Attributes>::new(),
            None,
            true,
            false,
            &[],
        );
        assert_eq!(features.len(), 1);
    }

    #[test]
    fn a_citygml_3_document_is_rejected_by_the_v2_parser() {
        let three = MINIMAL_CITYGML_2.replace("citygml/2.0", "citygml/3.0");
        let mut parser = Parser::new(CityGmlVersion::V2);
        let url = Url::parse("file:///test.gml").unwrap();
        let err = parser
            .parse(three.as_bytes(), &url)
            .expect_err("a 3.0 document must not parse as 2.0");
        let msg = format!("{err}");
        // Pinned to the real message text (captured by running this test with
        // a temporary panic!("{msg}") and reading the failure output):
        // "CityModel root element doesn't match the expected CityGML version
        // V2: found tag core:CityModel". Assert a distinctive fragment of it
        // so this test fails if the mismatch stops being reported, rather
        // than settling for "the message is merely non-empty".
        assert!(
            msg.contains("doesn't match the expected CityGML version"),
            "the mismatch error must name the version it expected, got: {msg}"
        );
    }

    // A Building with two boundary surfaces of the same extractable type, so that
    // extracting them changes the feature count relative to the default (1 -> 2)
    // rather than merely relabeling the same single feature.
    const CITYGML_2_WITH_WALLS: &str = concat!(
        r#"<?xml version="1.0" encoding="UTF-8"?>"#,
        r#"<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0""#,
        r#" xmlns:bldg="http://www.opengis.net/citygml/building/2.0""#,
        r#" xmlns:gml="http://www.opengis.net/gml">"#,
        r#"<core:cityObjectMember><bldg:Building gml:id="b1">"#,
        r#"<bldg:function>1000</bldg:function>"#,
        r#"<bldg:boundedBy><bldg:WallSurface gml:id="wall1"/></bldg:boundedBy>"#,
        r#"<bldg:boundedBy><bldg:WallSurface gml:id="wall2"/></bldg:boundedBy>"#,
        r#"</bldg:Building></core:cityObjectMember>"#,
        r#"</core:CityModel>"#,
    );

    fn default_property() -> CityGml2Property {
        CityGml2Property {
            extract_tags: vec![],
            keep_attributes: true,
            flatten_measure_types: false,
            city_gml_attributes_key: None,
        }
    }

    async fn drain_features(rx: &mut mpsc::Receiver<(Port, IngestionMessage)>) -> usize {
        let mut count = 0;
        while let Ok((port, msg)) = rx.try_recv() {
            assert_eq!(port, *FEATURES_PORT, "the reader has one output port");
            assert!(matches!(msg, IngestionMessage::OperationEvent { .. }));
            count += 1;
        }
        count
    }

    /// `inline` has no path of its own, so `start()` must fall back to the
    /// synthetic `file:///inline.gml` source URL rather than failing or
    /// requiring `dataset`.
    #[tokio::test]
    async fn start_reads_inline_content_and_emits_one_feature_per_city_object() {
        let mut reader = CityGml2Reader {
            common: FileReaderCompiledParam {
                dataset: None,
                inline: Some(Bytes::from(MINIMAL_CITYGML_2)),
            },
            property: default_property(),
        };
        let (tx, mut rx) = mpsc::channel(16);
        reader
            .start(NodeContext::default(), tx)
            .await
            .expect("start should succeed reading inline content");
        assert_eq!(
            drain_features(&mut rx).await,
            1,
            "the single Building in MINIMAL_CITYGML_2 should arrive as one feature"
        );
    }

    /// `extract_tags` is threaded into both `Parser::with_extract_tags` and
    /// `build_features`; if either wiring were dropped, hoisting would not
    /// happen and the feature count would stay at the default.
    #[tokio::test]
    async fn start_with_extract_tags_hoists_matching_boundary_surfaces() {
        async fn run(extract_tags: Vec<String>) -> usize {
            let mut reader = CityGml2Reader {
                common: FileReaderCompiledParam {
                    dataset: None,
                    inline: Some(Bytes::from(CITYGML_2_WITH_WALLS)),
                },
                property: CityGml2Property {
                    extract_tags,
                    ..default_property()
                },
            };
            let (tx, mut rx) = mpsc::channel(16);
            reader
                .start(NodeContext::default(), tx)
                .await
                .expect("start should succeed");
            drain_features(&mut rx).await
        }

        let default_count = run(vec![]).await;
        let extracted_count = run(vec!["WallSurface".to_string()]).await;
        assert_eq!(
            default_count, 1,
            "without extract_tags, the Building and its two boundary surfaces collapse into one feature"
        );
        assert_eq!(
            extracted_count, 2,
            "with extract_tags including WallSurface, both boundary surfaces are hoisted out as their own features and the Building itself (not in the tag set) is dropped"
        );
    }

    // A Building whose one surface's `gml:posList` contains a token that isn't a
    // valid number. The XML itself is well-formed, so `Parser::parse` succeeds;
    // the malformation only surfaces once the geometry is resolved.
    const CITYGML_2_WITH_BAD_POSLIST: &str = concat!(
        r#"<?xml version="1.0" encoding="UTF-8"?>"#,
        r#"<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0""#,
        r#" xmlns:bldg="http://www.opengis.net/citygml/building/2.0""#,
        r#" xmlns:gml="http://www.opengis.net/gml">"#,
        r#"<core:cityObjectMember><bldg:Building gml:id="b1">"#,
        r#"<bldg:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>"#,
        r#"<gml:Polygon><gml:exterior><gml:LinearRing>"#,
        r#"<gml:posList>1.0 nope 3.0</gml:posList>"#,
        r#"</gml:LinearRing></gml:exterior></gml:Polygon>"#,
        r#"</gml:surfaceMember></gml:MultiSurface></bldg:lod2MultiSurface>"#,
        r#"</bldg:Building></core:cityObjectMember>"#,
        r#"</core:CityModel>"#,
    );

    /// Action Standard §4.3: present-but-malformed input must fail the read
    /// naming the offending location, not emit a feature with its geometry
    /// silently dropped. Checked before the send loop, so nothing reaches the
    /// channel. Only the new-geometry path is instrumented for this (the
    /// legacy path is not; see `reearth_flow_citygml::pipeline`).
    #[cfg(feature = "new-geometry")]
    #[tokio::test]
    async fn start_fails_the_read_on_malformed_poslist_and_sends_no_features() {
        let mut reader = CityGml2Reader {
            common: FileReaderCompiledParam {
                dataset: None,
                inline: Some(Bytes::from(CITYGML_2_WITH_BAD_POSLIST)),
            },
            property: default_property(),
        };
        let (tx, mut rx) = mpsc::channel(16);
        let err = reader
            .start(NodeContext::default(), tx)
            .await
            .expect_err("a malformed gml:posList must fail the read");
        let msg = format!("{err}");
        assert!(
            msg.contains("invalid gml:posList content"),
            "the error must name the reason, got: {msg}"
        );
        assert_eq!(
            drain_features(&mut rx).await,
            0,
            "no features should reach the channel when the read fails"
        );
    }

    /// The same malformed document through `build_features` (what the
    /// `Feature CityGML 2 Reader` processor calls) must stay lenient: it
    /// returns the feature, geometry silently dropped, and does not error.
    #[test]
    fn build_features_stays_lenient_on_the_same_malformed_poslist() {
        let mut parser = Parser::new(CityGmlVersion::V2);
        let url = Url::parse("file:///test.gml").unwrap();
        parser
            .parse(CITYGML_2_WITH_BAD_POSLIST.as_bytes(), &url)
            .expect("XML is well-formed even though the posList content is not");
        let features = build_features(
            parser,
            &HashSet::new(),
            &HashMap::<String, Attributes>::new(),
            None,
            true,
            false,
            &[],
        );
        assert_eq!(
            features.len(),
            1,
            "the processor path must stay lenient and still emit the Building"
        );
    }
}
