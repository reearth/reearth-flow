use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::Arc,
};

use quick_xml::{
    events::{BytesStart, Event},
    name::{Namespace, ResolveResult},
    NsReader,
};
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Attributes, Code, CompiledCode, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

use super::errors::PlateauProcessorError;
use super::PlateauProfile;

static GML_NAME_ERRORS_PORT: once_cell::sync::Lazy<Port> =
    once_cell::sync::Lazy::new(|| Port::new("gmlNameErrors"));

static STATS_PORT: once_cell::sync::Lazy<Port> = once_cell::sync::Lazy::new(|| Port::new("stats"));

/// # GmlNameCodeSpaceValidator Parameters
///
/// Configuration for validating gml:name elements to ensure they have codeSpace attributes.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct GmlNameCodeSpaceValidatorParam {
    /// Expression to get the path to the CityGML file
    #[serde(skip_serializing_if = "Option::is_none")]
    city_gml_path: Option<Code>,
}

/// Reads each CityGML file once and reports every generic city object whose
/// name is not coded.
///
/// A generic city object is checked when it is not nested in another one. Only
/// the `gml:name` elements directly under it are looked at, so a name of a
/// nested object is not counted against the object that holds it. A
/// `gml:name` without a `codeSpace` attribute is reported once, with its value.
/// An object with no `gml:name` at all is reported once, with an empty value.
///
/// `gmlNameErrors` carries one feature per finding with `Folder`, `Index`,
/// `Filename`, `gml_id` and `コード化されていないgml:name`. `stats` carries one
/// feature per file with `Filename` and `_num_uncoded_gml_name`. Every input
/// feature is passed on unchanged through `features`.
#[derive(Debug, Clone)]
pub(crate) struct GmlNameCodeSpaceValidatorFactory {
    name: String,
    profile: &'static PlateauProfile,
}

impl GmlNameCodeSpaceValidatorFactory {
    pub(crate) fn new(profile: &'static PlateauProfile) -> Self {
        Self {
            name: profile.action_name("GmlNameCodeSpaceValidator"),
            profile,
        }
    }
}

impl ProcessorFactory for GmlNameCodeSpaceValidatorFactory {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Validates that gml:name elements have codeSpace attributes (coded values)"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GmlNameCodeSpaceValidatorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            FEATURES_PORT.clone(),
            GML_NAME_ERRORS_PORT.clone(),
            STATS_PORT.clone(),
        ]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: GmlNameCodeSpaceValidatorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::GmlNameCodeSpaceValidatorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::GmlNameCodeSpaceValidatorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            GmlNameCodeSpaceValidatorParam::default()
        };

        let city_gml_path_expr = params
            .city_gml_path
            .map(|expr| {
                expr.compile().map_err(|e| {
                    PlateauProcessorError::GmlNameCodeSpaceValidatorFactory(format!(
                        "Failed to compile cityGmlPath expression: {e}"
                    ))
                })
            })
            .transpose()?;

        let process = GmlNameCodeSpaceValidator {
            targets: TargetElements::from_profile(self.profile)?,
            city_gml_path_expr,
            processed_files: HashMap::new(),
            processed_paths: HashSet::new(),
        };
        Ok(Box::new(process))
    }
}

/// The generic city object elements of one PLATEAU generation.
#[derive(Debug, Clone)]
struct TargetElements {
    namespace: &'static str,
    local_names: &'static [&'static str],
    gml_namespace: &'static str,
}

impl TargetElements {
    fn from_profile(profile: &PlateauProfile) -> Result<Self, PlateauProcessorError> {
        let namespace = |prefix: &str| {
            profile.citygml.namespace(prefix).ok_or_else(|| {
                PlateauProcessorError::GmlNameCodeSpaceValidatorFactory(format!(
                    "No namespace for `{prefix}`"
                ))
            })
        };
        Ok(Self {
            namespace: namespace("gen")?,
            local_names: profile.generic_city_object_types(),
            gml_namespace: namespace("gml")?,
        })
    }

    fn is_target(&self, ns: &ResolveResult, local_name: &[u8]) -> bool {
        is_in_namespace(ns, self.namespace)
            && self
                .local_names
                .iter()
                .any(|name| name.as_bytes() == local_name)
    }

    fn is_gml_name(&self, ns: &ResolveResult, local_name: &[u8]) -> bool {
        is_in_namespace(ns, self.gml_namespace) && local_name == b"name"
    }
}

fn is_in_namespace(ns: &ResolveResult, namespace: &str) -> bool {
    matches!(ns, ResolveResult::Bound(Namespace(uri)) if *uri == namespace.as_bytes())
}

#[derive(Debug, Clone, PartialEq)]
struct UncodedGmlNameError {
    gml_id: String,
    gml_name: String,
}

#[derive(Debug, Clone, Default)]
struct FileStats {
    filename: String,
    error_count: usize,
}

#[derive(Debug, Clone)]
struct GmlNameCodeSpaceValidator {
    targets: TargetElements,
    city_gml_path_expr: Option<CompiledCode>,
    processed_files: HashMap<String, FileStats>,
    /// Paths already read, so a file several features come from is read once.
    processed_paths: HashSet<String>,
}

impl Processor for GmlNameCodeSpaceValidator {
    fn num_threads(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        let city_gml_path = if let Some(expr) = &self.city_gml_path_expr {
            expr.eval_string(feature, ctx.variables.clone())
                .map_err(|e| {
                    PlateauProcessorError::GmlNameCodeSpaceValidator(format!(
                        "Failed to evaluate cityGmlPath expression: {e:?}"
                    ))
                })?
        } else {
            feature
                .get(Attribute::new("gmlPath"))
                .and_then(|v| match v {
                    AttributeValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .ok_or_else(|| {
                    PlateauProcessorError::GmlNameCodeSpaceValidator(
                        "No cityGmlPath expression and no gmlPath attribute found".to_string(),
                    )
                })?
        };

        let folder = feature
            .get(Attribute::new("udxDirs"))
            .and_then(|v| match v {
                AttributeValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "gen".to_string());

        let file_index = feature
            .get(Attribute::new("fileIndex"))
            .and_then(|v| match v {
                AttributeValue::Number(n) => n.as_u64(),
                AttributeValue::String(s) => s.parse().ok(),
                _ => None,
            })
            .unwrap_or(1);

        let filename = feature
            .get(Attribute::new("gmlFilename"))
            .and_then(|v| match v {
                AttributeValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| {
                city_gml_path
                    .split('/')
                    .next_back()
                    .unwrap_or("unknown.gml")
                    .to_string()
            });

        if self.processed_paths.insert(city_gml_path.clone()) {
            let errors = self.validate_gml_names(&ctx, &city_gml_path)?;

            let stats = self
                .processed_files
                .entry(filename.clone())
                .or_insert_with(|| FileStats {
                    filename: filename.clone(),
                    error_count: 0,
                });
            stats.error_count += errors.len();

            for error in errors {
                let mut error_feature = Feature::new_with_attributes(Attributes::new());
                error_feature.insert("Folder", AttributeValue::String(folder.clone()));
                error_feature.insert("Index", AttributeValue::Number(Number::from(file_index)));
                error_feature.insert("Filename", AttributeValue::String(filename.clone()));
                error_feature.insert("gml_id", AttributeValue::String(error.gml_id));
                error_feature.insert(
                    "コード化されていないgml:name",
                    AttributeValue::String(error.gml_name),
                );

                fw.send(ctx.new_with_feature_and_port(error_feature, GML_NAME_ERRORS_PORT.clone()));
            }
        }

        fw.send(ctx.new_with_feature_and_port(feature.clone(), FEATURES_PORT.clone()));

        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        for stats in self.processed_files.values() {
            let mut stats_feature = Feature::new_with_attributes(Attributes::new());
            stats_feature.insert("Filename", AttributeValue::String(stats.filename.clone()));
            stats_feature.insert(
                "_num_uncoded_gml_name",
                AttributeValue::Number(Number::from(stats.error_count)),
            );

            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                stats_feature,
                STATS_PORT.clone(),
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "GmlNameCodeSpaceValidator"
    }
}

impl GmlNameCodeSpaceValidator {
    fn validate_gml_names(
        &self,
        ctx: &ExecutorContext,
        city_gml_path: &str,
    ) -> super::errors::Result<Vec<UncodedGmlNameError>> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);

        let city_gml_uri = Uri::from_str(city_gml_path).map_err(|e| {
            PlateauProcessorError::GmlNameCodeSpaceValidator(format!("Invalid URI: {e:?}"))
        })?;

        let storage = storage_resolver.resolve(&city_gml_uri).map_err(|e| {
            PlateauProcessorError::GmlNameCodeSpaceValidator(format!(
                "Failed to resolve storage: {e:?}"
            ))
        })?;

        let xml_content = storage
            .get_sync(city_gml_uri.path().as_path())
            .map_err(|e| {
                PlateauProcessorError::GmlNameCodeSpaceValidator(format!(
                    "Failed to read file: {e:?}"
                ))
            })?;

        find_uncoded_gml_names(&xml_content, &self.targets)
    }
}

/// The generic city object being read and whether a `gml:name` was seen
/// directly under it.
struct OpenTarget {
    depth: usize,
    gml_id: String,
    has_name: bool,
}

fn find_uncoded_gml_names(
    xml: &[u8],
    targets: &TargetElements,
) -> super::errors::Result<Vec<UncodedGmlNameError>> {
    let parse_error = |e: quick_xml::Error| {
        PlateauProcessorError::GmlNameCodeSpaceValidator(format!("Failed to parse XML: {e}"))
    };
    let mut reader = NsReader::from_reader(xml);
    let mut buf = Vec::new();
    let mut errors = Vec::new();
    let mut depth = 0usize;
    let mut open: Option<OpenTarget> = None;

    loop {
        let (ns, event) = reader
            .read_resolved_event_into(&mut buf)
            .map_err(parse_error)?;
        match event {
            Event::Start(ref e) | Event::Empty(ref e) => {
                let is_empty = matches!(event, Event::Empty(_));
                depth += 1;
                let local_name = e.local_name();
                match open.as_mut() {
                    None if targets.is_target(&ns, local_name.as_ref()) => {
                        let target = OpenTarget {
                            depth,
                            gml_id: gml_id(&reader, e, targets)?,
                            has_name: false,
                        };
                        if is_empty {
                            errors.push(UncodedGmlNameError {
                                gml_id: target.gml_id,
                                gml_name: String::new(),
                            });
                        } else {
                            open = Some(target);
                        }
                    }
                    Some(target)
                        if depth == target.depth + 1
                            && targets.is_gml_name(&ns, local_name.as_ref()) =>
                    {
                        target.has_name = true;
                        let has_code_space = e
                            .attributes()
                            .flatten()
                            .any(|attr| attr.key.as_ref() == b"codeSpace");
                        let value = if is_empty {
                            String::new()
                        } else {
                            let end = e.to_end().into_owned();
                            let text = reader.read_text(end.name()).map_err(parse_error)?;
                            depth -= 1;
                            quick_xml::escape::unescape(&text)
                                .map_err(|e| parse_error(e.into()))?
                                .into_owned()
                        };
                        if !has_code_space {
                            errors.push(UncodedGmlNameError {
                                gml_id: target.gml_id.clone(),
                                gml_name: value,
                            });
                        }
                    }
                    _ => {}
                }
                if is_empty {
                    depth -= 1;
                }
            }
            Event::End(_) => {
                if let Some(target) = open.take_if(|target| target.depth == depth) {
                    if !target.has_name {
                        errors.push(UncodedGmlNameError {
                            gml_id: target.gml_id,
                            gml_name: String::new(),
                        });
                    }
                }
                depth -= 1;
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(errors)
}

fn gml_id(
    reader: &NsReader<&[u8]>,
    element: &BytesStart,
    targets: &TargetElements,
) -> super::errors::Result<String> {
    for attr in element.attributes().flatten() {
        let (ns, local_name) = reader.resolve_attribute(attr.key);
        if local_name.as_ref() == b"id" && is_in_namespace(&ns, targets.gml_namespace) {
            let value = attr.unescape_value().map_err(|e| {
                PlateauProcessorError::GmlNameCodeSpaceValidator(format!(
                    "Failed to read gml:id: {e}"
                ))
            })?;
            return Ok(value.into_owned());
        }
    }
    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const GEN2: &str = r#"xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:gml="http://www.opengis.net/gml""#;
    const GEN3: &str = r#"xmlns:gen="http://www.opengis.net/citygml/generics/3.0" xmlns:core="http://www.opengis.net/citygml/3.0" xmlns:gml="http://www.opengis.net/gml/3.2""#;

    static TEST_PROFILE_CITYGML3: PlateauProfile = PlateauProfile {
        citygml: &crate::citygml::CITYGML3,
        uro_ns: "https://www.geospatial.jp/iur/uro/4.0",
        urf_ns: "https://www.geospatial.jp/iur/urf/4.0",
        action_prefix: "PLATEAU6",
    };

    fn plateau4() -> TargetElements {
        TargetElements::from_profile(&crate::plateau4::PLATEAU4).unwrap()
    }

    fn plateau6() -> TargetElements {
        TargetElements::from_profile(&TEST_PROFILE_CITYGML3).unwrap()
    }

    fn error(gml_id: &str, gml_name: &str) -> UncodedGmlNameError {
        UncodedGmlNameError {
            gml_id: gml_id.to_string(),
            gml_name: gml_name.to_string(),
        }
    }

    fn find(xml: &str, targets: &TargetElements) -> Vec<UncodedGmlNameError> {
        find_uncoded_gml_names(xml.as_bytes(), targets).unwrap()
    }

    #[test]
    fn test_factory_name() {
        let factory = GmlNameCodeSpaceValidatorFactory::new(&TEST_PROFILE_CITYGML3);
        assert_eq!(factory.name(), "PLATEAU6.GmlNameCodeSpaceValidator");
    }

    #[test]
    fn test_uncoded_and_coded_names() {
        let xml = format!(
            r#"<root {GEN2}>
              <gen:GenericCityObject gml:id="gen_coded">
                <gml:name codeSpace="../../codelists/test.xml">CodedValue</gml:name>
              </gen:GenericCityObject>
              <gen:GenericCityObject gml:id="gen_uncoded">
                <gml:name>UncodedValue</gml:name>
              </gen:GenericCityObject>
            </root>"#
        );
        assert_eq!(
            find(&xml, &plateau4()),
            vec![error("gen_uncoded", "UncodedValue")]
        );
    }

    #[test]
    fn test_object_without_name() {
        let xml = format!(
            r#"<root {GEN2}>
              <gen:GenericCityObject gml:id="gen_no_name">
                <gen:class>1</gen:class>
              </gen:GenericCityObject>
              <gen:GenericCityObject gml:id="gen_empty"/>
            </root>"#
        );
        assert_eq!(
            find(&xml, &plateau4()),
            vec![error("gen_no_name", ""), error("gen_empty", "")]
        );
    }

    #[test]
    fn test_prefix_does_not_matter() {
        let xml = r#"<root xmlns:g="http://www.opengis.net/citygml/generics/2.0" xmlns:n="http://www.opengis.net/gml">
              <g:GenericCityObject n:id="gen_1"><n:name>A&amp;B</n:name></g:GenericCityObject>
            </root>"#;
        assert_eq!(find(xml, &plateau4()), vec![error("gen_1", "A&B")]);
    }

    #[test]
    fn test_other_generation_is_ignored() {
        let xml = format!(
            r#"<root {GEN3}>
              <gen:GenericOccupiedSpace gml:id="gen_1"><gml:name>20</gml:name></gen:GenericOccupiedSpace>
            </root>"#
        );
        assert_eq!(find(&xml, &plateau4()), vec![]);
        assert_eq!(find(&xml, &plateau6()), vec![error("gen_1", "20")]);
    }

    #[test]
    fn test_every_citygml3_type() {
        let xml = format!(
            r#"<root {GEN3}>
              <gen:GenericOccupiedSpace gml:id="a"><gml:name>1</gml:name></gen:GenericOccupiedSpace>
              <gen:GenericUnoccupiedSpace gml:id="b"><gml:name>2</gml:name></gen:GenericUnoccupiedSpace>
              <gen:GenericLogicalSpace gml:id="c"><gml:name>3</gml:name></gen:GenericLogicalSpace>
              <gen:GenericThematicSurface gml:id="d"><gml:name>4</gml:name></gen:GenericThematicSurface>
            </root>"#
        );
        assert_eq!(
            find(&xml, &plateau6()),
            vec![
                error("a", "1"),
                error("b", "2"),
                error("c", "3"),
                error("d", "4")
            ]
        );
    }

    #[test]
    fn test_nested_object_is_not_counted() {
        let xml = format!(
            r#"<root {GEN3}>
              <gen:GenericOccupiedSpace gml:id="outer">
                <gml:name codeSpace="../../codelists/GenericCityObject_name.xml">20</gml:name>
                <core:boundary>
                  <gen:GenericThematicSurface gml:id="inner">
                    <gml:name>30</gml:name>
                  </gen:GenericThematicSurface>
                </core:boundary>
              </gen:GenericOccupiedSpace>
            </root>"#
        );
        assert_eq!(find(&xml, &plateau6()), vec![]);
    }

    #[test]
    fn test_name_of_nested_element_does_not_count_as_name() {
        let xml = format!(
            r#"<root {GEN3}>
              <gen:GenericOccupiedSpace gml:id="outer">
                <core:boundary>
                  <gen:GenericThematicSurface gml:id="inner">
                    <gml:name codeSpace="../../codelists/GenericCityObject_name.xml">20</gml:name>
                  </gen:GenericThematicSurface>
                </core:boundary>
              </gen:GenericOccupiedSpace>
            </root>"#
        );
        assert_eq!(find(&xml, &plateau6()), vec![error("outer", "")]);
    }
}
