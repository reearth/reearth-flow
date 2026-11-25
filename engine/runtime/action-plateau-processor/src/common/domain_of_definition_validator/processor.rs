use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use fastxml::parser;
use fastxml::schema::types::CompiledSchema;
use fastxml::transform::{EditableNodeRef, TransformError, Transformer};
use reearth_flow_common::uri::Uri;
use reearth_flow_diagnostics::{DiagnosticDraft, ErrorCode};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{FEATURES_PORT, Port, Processor, ProcessorFactory, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Code, CompiledCode, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::code_value::{
    CodeValueCheck, CodelistResolver, Codelists, LoadedCodelists, create_codelist,
    get_dir_codelists,
};
use super::error::{self, Result};
use super::extents::{ExtentsCheck, stream_extract_envelope};
use super::feature_type::{FeatureTypeCheck, valid_feature_types};
use super::gml_id_format::GmlIdFormatCheck;
use super::gml_id_uniqueness::{
    DUPLICATE_GML_ID_STATS_PORT, FileNames, GmlIdTable, GmlIdUniqueness, member_gml_ids,
    merge_member,
};
use super::lod_geometry_type::LodGeometryTypeCheck;
use super::report::Reporter;
use super::srs_name::SrsNameCheck;
use super::traversal::collect_descendant_info;
use super::xlink::{self, ExternalGmlIds, XlinkCheck};
use crate::common::PlateauProfile;
use crate::common::application_schema_conformance::{
    CityObjectTypes, ConformanceRules, ElementName, element_name,
};
use crate::common::errors::PlateauProcessorError;
use crate::common::object_list_extractor::OBJECT_LIST_DEFINITIONS_ATTRIBUTE;
use crate::object_list::ObjectListDefinitions;

/// An error of processing an input feature, other than those of the checks.
#[derive(thiserror::Error, Debug)]
pub(super) enum Error {
    #[error("the input feature has no `{0}` attribute")]
    MissingAttribute(&'static str),
    #[error("the `package` attribute is not a string")]
    PackageNotString,
    #[error("failed to evaluate codelistsPath: {0}")]
    CodelistsPathEval(#[source] reearth_flow_types::error::Error),
    #[error("invalid codelistsPath URI {path}: {source}")]
    CodelistsPathUri {
        path: String,
        #[source]
        source: reearth_flow_common::Error,
    },
    #[error("invalid objectListDefinitions: {0}")]
    ObjectListDefinitions(#[source] serde_json::Error),
    #[error("invalid city GML path {path}: {source}")]
    CityGmlPath {
        path: String,
        #[source]
        source: reearth_flow_common::Error,
    },
    #[error("failed to resolve the storage of the city GML {path}: {source}")]
    CityGmlStorage {
        path: String,
        #[source]
        source: reearth_flow_storage::Error,
    },
    #[error("failed to read the city GML {path}: {source}")]
    CityGmlRead {
        path: String,
        #[source]
        source: object_store::Error,
    },
    #[error("the city GML {path} is not UTF-8: {source}")]
    CityGmlNotUtf8 {
        path: String,
        #[source]
        source: std::str::Utf8Error,
    },
    #[error("failed to parse the city GML: {0}")]
    CityGmlParse(#[source] TransformError),
    #[error("a cityObjectMember has no child element")]
    EmptyMember,
    #[error("a city object has no gml:id")]
    MissingGmlId,
}

/// # DomainOfDefinitionValidator Parameters
///
/// Configuration for validating domain of definition of CityGML features.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct DomainOfDefinitionValidatorParam {
    /// # Codelists Path
    /// Expression for the URI of a directory of codelist files, evaluated for each
    /// input feature. The codelists are read from it when the first input feature has
    /// no `dirCodelists` attribute or its directory does not exist; without either,
    /// the action fails. A codeSpace found neither under `dirCodelists` nor relative
    /// to the city GML is looked up in it by the file name of the codeSpace.
    #[serde(skip_serializing_if = "Option::is_none")]
    codelists_path: Option<Code>,
}

#[derive(Debug, Clone)]
pub(crate) struct DomainOfDefinitionValidatorFactory {
    profile: &'static PlateauProfile,
    name: String,
}

impl DomainOfDefinitionValidatorFactory {
    pub(crate) fn new(profile: &'static PlateauProfile) -> Self {
        Self {
            name: profile.action_name("DomainOfDefinitionValidator"),
            profile,
        }
    }
}

impl ProcessorFactory for DomainOfDefinitionValidatorFactory {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Validates domain of definition of CityGML features"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(DomainOfDefinitionValidatorParam))
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
            REJECTED_PORT.clone(),
            DUPLICATE_GML_ID_STATS_PORT.clone(),
        ]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: DomainOfDefinitionValidatorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::DomainOfDefinitionValidatorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::DomainOfDefinitionValidatorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            DomainOfDefinitionValidatorParam::default()
        };

        let codelists_path_expr = params
            .codelists_path
            .map(|expr| {
                expr.compile().map_err(|e| {
                    PlateauProcessorError::DomainOfDefinitionValidatorFactory(format!(
                        "Failed to compile codelists_path expression: {e}"
                    ))
                })
            })
            .transpose()?;

        let process = DomainOfDefinitionValidator {
            profile: self.profile,
            gml_id_uniqueness: GmlIdUniqueness::default(),
            codelists: None,
            caches: SharedCaches::default(),
            codelists_path_expr,
            conformance_rules: None,
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct DomainOfDefinitionValidator {
    profile: &'static PlateauProfile,
    gml_id_uniqueness: GmlIdUniqueness,
    codelists: Option<Codelists>,
    caches: SharedCaches,
    codelists_path_expr: Option<CompiledCode>,
    /// L03 objectlist conformance rules, read from the first feature. `Some(None)` when it
    /// has no objectlist definitions and L03 checks feature types only.
    conformance_rules: Option<Option<ConformanceRules>>,
}

impl Processor for DomainOfDefinitionValidator {
    fn num_threads(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        self.process_input(&ctx, fw)
            .map_err(PlateauProcessorError::from)?;
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        self.gml_id_uniqueness.finish(&ctx, fw);
        Ok(())
    }

    fn name(&self) -> &str {
        "DomainOfDefinitionValidator"
    }
}

impl DomainOfDefinitionValidator {
    /// Checks the city GML of the input feature of `ctx`.
    fn process_input(
        &mut self,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<()> {
        let feature = &ctx.feature;

        // Collect file name for statistics output
        if let Some(AttributeValue::String(name)) = feature.attributes.get(&Attribute::new("name"))
        {
            self.gml_id_uniqueness.add_filename(name.clone());
        }

        // Evaluate fallback codelists path expression if provided
        let fallback_codelists_path = if let Some(expr) = &self.codelists_path_expr {
            let path = expr
                .eval_string(feature, ctx.variables.clone())
                .map_err(Error::CodelistsPathEval)?;
            Some(Uri::from_str(&path).map_err(|source| Error::CodelistsPathUri { path, source })?)
        } else {
            None
        };

        if self.codelists.is_none() {
            let codelists = create_codelist(
                Arc::clone(&ctx.storage_resolver),
                feature,
                fallback_codelists_path.as_ref(),
            )?;
            self.codelists = Some(codelists);
        }
        if self.conformance_rules.is_none() {
            let rules = feature
                .get(OBJECT_LIST_DEFINITIONS_ATTRIBUTE)
                .map(|value| {
                    ObjectListDefinitions::try_from(value.clone())
                        .map_err(Error::ObjectListDefinitions)
                })
                .transpose()?
                .map(|definitions| ConformanceRules::new(definitions, self.profile));
            if let Some(unknown) = rules
                .as_ref()
                .map(ConformanceRules::unknown_prefixes)
                .filter(|unknown| !unknown.is_empty())
            {
                ctx.warn(
                    DiagnosticDraft::new(ErrorCode::PlateauObjectlistPrefixUnknown).with_message(
                        format!("objectlist prefixes without a namespace URI: {unknown:?}"),
                    ),
                );
            }
            self.conformance_rules = Some(rules);
        }
        let codelists = self.codelists.as_ref().unwrap();
        let conformance_rules = self.conformance_rules.as_ref().and_then(Option::as_ref);
        let inputs = ProcessorInputs {
            profile: self.profile,
            codelists,
            conformance_rules,
        };
        let gml_ids = process_feature(
            &inputs,
            ctx,
            fw,
            feature,
            fallback_codelists_path.as_ref(),
            &mut self.caches,
        )?;
        self.gml_id_uniqueness.add_file_gml_ids(gml_ids);
        Ok(())
    }
}

/// The inputs of the checks shared by all files.
struct ProcessorInputs<'a> {
    profile: &'a PlateauProfile,
    codelists: &'a Codelists,
    conformance_rules: Option<&'a ConformanceRules>,
}

/// What the checks of a file read from other files, kept for all files.
#[derive(Debug, Clone, Default)]
struct SharedCaches {
    /// The codelists not in `codelists`, loaded when a file refers to them.
    loaded_codelists: LoadedCodelists,
    /// The gml:ids of the GML files the xlinks refer to.
    external_gml_ids: ExternalGmlIds,
}

/// Read-only inputs of the checks of a file.
struct FileContext<'a> {
    profile: &'a PlateauProfile,
    ctx: &'a ExecutorContext,
    /// The attributes of the input feature an occurrence of a gml:id reports.
    file_names: Arc<FileNames>,
    /// The directory of the city GML; `None` when it has none.
    city_gml_dir: Option<&'a Uri>,
    reporter: Reporter<'a>,
    codelist_resolver: CodelistResolver<'a>,
}

/// The checks of a file, other than L03, and their state.
struct FileChecks<'a> {
    srs_name: SrsNameCheck,
    gml_id_format: GmlIdFormatCheck,
    code_value: CodeValueCheck<'a>,
    extents: ExtentsCheck,
    xlink: XlinkCheck<'a>,
    lod_geometry_type: LodGeometryTypeCheck,
    /// The gml:ids of the file.
    gml_ids: GmlIdTable,
    member_count: usize,
}

/// The schemas the city GML names in `xsi:schemaLocation`, or `None` when they
/// cannot be compiled.
fn application_schema(
    ctx: &ExecutorContext,
    xml: &[u8],
    city_gml_path: &Path,
) -> Option<Arc<CompiledSchema>> {
    let warn = |message: String| {
        ctx.warn(
            DiagnosticDraft::new(ErrorCode::PlateauApplicationSchemaUnavailable)
                .with_message(message),
        );
    };
    let locations = match parser::parse_schema_locations_from_reader(Cursor::new(xml)) {
        Ok(locations) if !locations.is_empty() => locations,
        Ok(_) => return None,
        Err(e) => {
            warn(format!("failed to read schema locations: {e:?}"));
            return None;
        }
    };
    let base_dir = city_gml_path.parent().filter(|dir| dir.exists());
    reearth_flow_http::compiled_schema(&locations, base_dir)
        .map(|compilation| Arc::clone(&compilation.schema))
        .inspect_err(|e| warn(format!("failed to compile schemas: {e}")))
        .ok()
}

/// Checks the city GML of `feature`, sends its errors and summary, and
/// returns its gml:ids.
fn process_feature(
    inputs: &ProcessorInputs,
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
    feature: &Feature,
    fallback_codelists_path: Option<&Uri>,
    caches: &mut SharedCaches,
) -> Result<GmlIdTable> {
    let ProcessorInputs {
        profile,
        codelists,
        conformance_rules,
    } = *inputs;
    let package = feature
        .attributes
        .get(&Attribute::new("package"))
        .ok_or(Error::MissingAttribute("package"))?;
    let AttributeValue::String(package) = package else {
        return Err(Error::PackageNotString.into());
    };
    let gml_id_format = GmlIdFormatCheck::new(package);
    let valid_feature_types = valid_feature_types(profile, package);

    let city_gml_path = feature
        .attributes
        .get(&Attribute::new("path"))
        .ok_or(Error::MissingAttribute("path"))?;

    let path = city_gml_path.to_string();
    let city_gml_uri = Uri::from_str(&path).map_err(|source| Error::CityGmlPath {
        path: path.clone(),
        source,
    })?;
    let storage = ctx
        .storage_resolver
        .resolve(&city_gml_uri)
        .map_err(|source| Error::CityGmlStorage {
            path: path.clone(),
            source,
        })?;
    let xml_content = storage
        .get_sync(city_gml_uri.path().as_path())
        .map_err(|source| Error::CityGmlRead {
            path: path.clone(),
            source,
        })?;
    let xml_str = std::str::from_utf8(&xml_content)
        .map_err(|source| Error::CityGmlNotUtf8 { path, source })?;

    // L03 needs the XSDs only for a package the objectlist has.
    let schema = (xlink::needs_application_schema(profile)
        || conformance_rules.is_some_and(|rules| rules.package(package, None).is_some()))
    .then(|| application_schema(ctx, &xml_content, &city_gml_uri.path()))
    .flatten();
    let city_gml_dir = city_gml_uri.dir();
    let city_objects = schema
        .as_deref()
        .map(|schema| CityObjectTypes::new(schema, profile.citygml.core_ns));

    let t_total = Instant::now();

    // Parse the city GML and run the per-member domain-of-definition checks.
    // If the document is not well-formed (e.g. an L01 syntax error), or a
    // member cannot be checked, the checks stop. Instead of dropping the file
    // from the result set, emit a Summary feature with the findings before the
    // stop (none for a document that is not well-formed), so the file still
    // appears in the per-file summary (01_共通.csv) and the error-count summary
    // (summary_common.json). The well-formedness problem itself is reported
    // separately as L01 by the upstream XMLValidator.
    let context = FileContext {
        profile,
        ctx,
        file_names: FileNames::new(feature),
        city_gml_dir: city_gml_dir.as_ref(),
        reporter: Reporter::new(ctx, fw, feature),
        codelist_resolver: CodelistResolver {
            codelists,
            storage_resolver: &ctx.storage_resolver,
            dir_codelists: get_dir_codelists(feature),
            city_gml_dir: city_gml_dir.as_ref(),
            fallback_codelists_path,
        },
    };
    let feature_type = RefCell::new(FeatureTypeCheck::new(
        valid_feature_types,
        conformance_rules.and_then(|rules| rules.package(package, city_objects)),
        conformance_rules.is_some(),
    ));
    let mut checks = FileChecks {
        srs_name: SrsNameCheck::default(),
        gml_id_format,
        code_value: CodeValueCheck::new(&mut caches.loaded_codelists),
        extents: ExtentsCheck::default(),
        xlink: XlinkCheck::new(&mut caches.external_gml_ids, profile, schema.as_deref()),
        lod_geometry_type: LodGeometryTypeCheck::default(),
        gml_ids: GmlIdTable::new(),
        member_count: 0,
    };
    let parse_outcome = run_checks(&context, xml_str, &feature_type, &mut checks);

    checks.xlink.warn_unresolved_associations(ctx);

    let parse_failed = parse_outcome.is_err();
    if let Err(e) = parse_outcome {
        tracing::debug!(
            error = ?e,
            "DomainOfDefinitionValidator: stopped checking the city GML; the summary has the findings before the stop"
        );
    }

    let total_ms = t_total.elapsed().as_millis();
    tracing::debug!(
        target: "perf",
        total_ms = %total_ms,
        member_count = checks.member_count,
        "DomainOfDefinitionValidator::process_feature END"
    );

    let mut summary = context.reporter.file_feature(&["index"]);
    summary.insert("flag", AttributeValue::String("Summary".to_string()));
    checks.srs_name.write_srs_name(&mut summary);
    feature_type.borrow().write_summary(&mut summary);
    checks.code_value.write_summary(&mut summary);
    checks
        .srs_name
        .write_validity(&mut summary, package, parse_failed);
    checks.extents.write_summary(&mut summary);
    checks.gml_id_format.write_summary(&mut summary);
    checks.xlink.write_summary(&mut summary);
    checks.lod_geometry_type.write_summary(&mut summary);
    context.reporter.send(summary);
    Ok(checks.gml_ids)
}

/// Parses the city GML and runs the per-`cityObjectMember` checks. Returns
/// `Err` if the document cannot be parsed (e.g. an L01 not-well-formed error)
/// or a member cannot be processed, in which case the remaining members and
/// the city object group xlinks are not checked.
fn run_checks(
    context: &FileContext,
    xml_str: &str,
    feature_type: &RefCell<FeatureTypeCheck>,
    checks: &mut FileChecks,
) -> Result<()> {
    let citygml = context.profile.citygml;
    let gml_ns = citygml.gml_ns;
    let envelope_xpath = format!(
        "//*[namespace-uri()='{}'][local-name()='Envelope']",
        citygml.gml_ns
    );
    let members_xpath = format!(
        "//*[namespace-uri()='{}'][local-name()='cityObjectMember']",
        citygml.core_ns
    );
    let appearance_members_xpath = format!(
        "//*[namespace-uri()='{}'][local-name()='appearanceMember']",
        citygml.core_ns
    );

    let bounded_by = stream_extract_envelope(xml_str, &envelope_xpath)?;
    checks.srs_name = SrsNameCheck::new(bounded_by.srs_name);
    checks.extents = ExtentsCheck::new(bounded_by.envelope);

    let transformer = Transformer::from(xml_str)
        .with_root_namespaces()
        .map_err(Error::CityGmlParse)?;

    let mut stream_error: Option<error::Error> = None;
    transformer
        .on(members_xpath.as_str(), |node| {
            checks.member_count += 1;
            if stream_error.is_some() {
                return;
            }

            let children = node.children();
            let Some(member) = children.first() else {
                stream_error = Some(Error::EmptyMember.into());
                return;
            };

            let member_name = element_name(context.profile, member);
            let is_city_object_group = member_name.key == "grp:CityObjectGroup";
            // L03 is counted before any error of the other checks.
            feature_type.borrow_mut().check_member(
                member,
                &member_name,
                is_city_object_group,
                gml_ns,
            );
            if let Err(e) =
                check_member(context, member, &member_name, is_city_object_group, checks)
            {
                stream_error = Some(e);
            }
        })
        .on(appearance_members_xpath.as_str(), |node| {
            feature_type
                .borrow_mut()
                .check_appearance_member(node, gml_ns);
        })
        .for_each()
        .map_err(Error::CityGmlParse)?;

    feature_type
        .borrow_mut()
        .report_violations(&context.reporter);

    if let Some(err) = stream_error {
        return Err(err);
    }

    checks
        .xlink
        .check_city_object_groups(&checks.gml_ids, &context.reporter);
    Ok(())
}

/// Runs the checks other than L03 on a member, in the order their errors are
/// sent.
fn check_member(
    context: &FileContext,
    member: &EditableNodeRef,
    member_name: &ElementName,
    is_city_object_group: bool,
    checks: &mut FileChecks,
) -> Result<()> {
    let gml_ns = context.profile.citygml.gml_ns;
    let Some(gml_id) = member.get_attribute_ns(gml_ns, "id") else {
        return Err(Error::MissingGmlId.into());
    };
    let feature_type = if member_name.is_standard {
        member.name()
    } else {
        "".to_string()
    };
    let report = context.reporter.member(&gml_id, &feature_type);
    let tag = member.qname();

    checks
        .gml_id_format
        .check_member(&gml_id, is_city_object_group, &tag, &report);

    let collected = collect_descendant_info(member, gml_ns, &tag, checks.extents.positions());
    let member_gml_ids =
        member_gml_ids(&context.file_names, &gml_id, &tag, collected.gml_id_entries);

    let result = (|| -> Result<()> {
        checks.code_value.check_member(
            &context.codelist_resolver,
            &collected.code_space_entries,
            &report,
        )?;

        checks.extents.check_member(collected.positions, &report)?;

        if is_city_object_group {
            checks
                .xlink
                .add_city_object_group(member.name(), gml_id, collected.xlinks.entries);
        } else {
            checks.xlink.check_member(
                &context.ctx.storage_resolver,
                gml_ns,
                context.city_gml_dir,
                &collected.xlinks,
                &member_gml_ids,
                &report,
            );
        }

        checks
            .lod_geometry_type
            .check_member(&collected.lod_entries, &report);
        Ok(())
    })();
    // The gml:ids are those of the file even when a check fails.
    merge_member(&mut checks.gml_ids, member_gml_ids);
    result
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use std::collections::{BTreeMap, HashSet};

    use reearth_flow_types::Attributes;
    use serde_json::Number;

    use super::super::extents::CoordinateError;
    use super::super::srs_name::{VALID_SRS_NAME_6668, VALID_SRS_NAME_6697};
    use crate::common::application_schema_conformance::Reason;
    use crate::object_list::FeatureTypeDefinition;
    use crate::tests::utils::{create_default_execute_context, create_default_node_context};
    use indexmap::IndexMap;
    use reearth_flow_runtime::{
        event::EventHub,
        forwarder::{NoopChannelForwarder, ProcessorChannelForwarder},
        node::ProcessorFactory,
    };
    use reearth_flow_types::Feature;
    use std::collections::HashMap;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_gml_file(file_path: &str, gml_id: &str) -> std::io::Result<()> {
        let gml_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd">
    <gml:boundedBy>
        <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
            <gml:lowerCorner>36.6470041354812 137.05268308385453 0</gml:lowerCorner>
            <gml:upperCorner>36.647798243275254 137.0537094956814 105.03314</gml:upperCorner>
        </gml:Envelope>
    </gml:boundedBy>
    <core:cityObjectMember>
        <bldg:Building gml:id="{gml_id}">
            <core:creationDate>2025-03-21</core:creationDate>
            <bldg:class codeSpace="../../codelists/Building_class.xml">3003</bldg:class>
            <bldg:usage codeSpace="../../codelists/Building_usage.xml">411</bldg:usage>
            <bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
            <bldg:measuredHeight uom="m">8.6</bldg:measuredHeight>
            <bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
            <bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
            <bldg:lod0RoofEdge>
                <gml:MultiSurface>
                    <gml:surfaceMember>
                        <gml:Polygon>
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>36.64773967207627 137.0537094956814 0 36.647798243275254 137.05370057460766 0 36.64778832538864 137.053600937653 0 36.64772975430324 137.053609970643 0 36.64773967207627 137.0537094956814 0</gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>
                </gml:MultiSurface>
            </bldg:lod0RoofEdge>
        </bldg:Building>
    </core:cityObjectMember>
</core:CityModel>"#
        );

        if let Some(parent) = std::path::Path::new(file_path).parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(file_path)?;
        file.write_all(gml_content.as_bytes())?;
        Ok(())
    }

    fn create_test_feature(
        name: &str,
        file_path: &str,
        codelists_dir: &std::path::Path,
    ) -> Feature {
        let mut attributes = IndexMap::new();
        attributes.insert(
            Attribute::new("name"),
            AttributeValue::String(name.to_string()),
        );
        attributes.insert(
            Attribute::new("package"),
            AttributeValue::String("bldg".to_string()),
        );
        attributes.insert(
            Attribute::new("dirCodelists"),
            AttributeValue::String(format!("file://{}", codelists_dir.display())),
        );
        attributes.insert(
            Attribute::new("path"),
            AttributeValue::String(file_path.to_string()),
        );

        Feature::new_with_attributes(attributes)
    }

    // Helper function to extract file_stats outputs from ProcessorChannelForwarder
    fn extract_file_stats_outputs(
        fw: &ProcessorChannelForwarder,
    ) -> Result<HashMap<String, u64>, BoxedError> {
        match fw {
            ProcessorChannelForwarder::Noop(noop_fw) => {
                let send_ports = noop_fw.send_ports.lock().unwrap();
                let send_features = noop_fw.send_features.lock().unwrap();

                let file_stats_outputs: HashMap<String, u64> = send_ports
                    .iter()
                    .enumerate()
                    .filter(|(_, port)| port.as_ref() == "duplicateGmlIdStats")
                    .map(|(i, _)| &send_features[i])
                    .filter_map(|feature| {
                        if let (
                            Some(AttributeValue::String(filename)),
                            Some(AttributeValue::Number(count)),
                        ) = (feature.get("filename"), feature.get("duplicateGmlIdCount"))
                        {
                            Some((filename.clone(), count.as_u64().unwrap_or(0)))
                        } else {
                            None
                        }
                    })
                    .collect();

                Ok(file_stats_outputs)
            }
            ProcessorChannelForwarder::ChannelManager(_) => {
                Err("Expected Noop forwarder for testing".into())
            }
        }
    }

    // Helper function to setup and run processor with given GML file configs
    fn run_processor_test(
        gml_configs: Vec<(&str, &str)>, // (filename, gml_id) pairs
    ) -> Result<HashMap<String, u64>, BoxedError> {
        let temp_dir = TempDir::with_prefix("domain_of_definition_validator_test_")?;

        let codelists_dir = temp_dir.path().join("codelists");
        fs::create_dir_all(&codelists_dir)?;

        // Create GML files and collect features
        let mut features = Vec::new();
        for (filename, gml_id) in gml_configs {
            let file_path = temp_dir.path().join(filename);
            create_gml_file(&file_path.to_string_lossy(), gml_id)?;
            features.push(create_test_feature(
                filename,
                &format!("file://{}", file_path.display()),
                &codelists_dir,
            ));
        }

        // Create processor (CityGML 2.0 fixture -> PLATEAU4 profile)
        let factory = DomainOfDefinitionValidatorFactory::new(&crate::plateau4::PLATEAU4);
        let ctx = create_default_node_context();
        let mut processor: Box<dyn Processor> =
            factory.build(ctx, EventHub::new(1024), "test".to_string(), None)?;

        // Process features
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        for feature in features {
            let ctx = create_default_execute_context(feature);
            processor.process(ctx, &fw)?;
        }

        // Call finish to trigger file_stats output
        let ctx = create_default_node_context();
        processor.finish(ctx, &fw)?;

        // Extract results
        let file_stats_outputs = extract_file_stats_outputs(&fw)?;

        Ok(file_stats_outputs)
    }

    #[test]
    fn test_gml_id_duplicate_detection() -> Result<(), BoxedError> {
        // Create test GML files:
        let gml_id_1 = format!("bldg_{}", uuid::Uuid::new_v4());
        let gml_id_2 = format!("bldg_{}", uuid::Uuid::new_v4());
        let gml_configs = vec![
            ("file1.gml", gml_id_1.as_str()),
            ("file2.gml", gml_id_1.as_str()),
            ("file3.gml", gml_id_2.as_str()),
        ];

        let file_stats_outputs = run_processor_test(gml_configs)?;

        // Assert the expected duplicate counts:
        assert_eq!(file_stats_outputs.get("file1.gml"), Some(&1));
        assert_eq!(file_stats_outputs.get("file2.gml"), Some(&1));
        assert_eq!(file_stats_outputs.get("file3.gml"), Some(&0));
        Ok(())
    }

    #[test]
    fn test_gml_id_no_duplicates() -> Result<(), BoxedError> {
        // Create test GML files with all unique gml:ids:
        let gml_id_1 = format!("bldg_{}", uuid::Uuid::new_v4());
        let gml_id_2 = format!("bldg_{}", uuid::Uuid::new_v4());
        let gml_id_3 = format!("bldg_{}", uuid::Uuid::new_v4());
        let gml_configs = vec![
            ("file1.gml", gml_id_1.as_str()),
            ("file2.gml", gml_id_2.as_str()),
            ("file3.gml", gml_id_3.as_str()),
        ];

        let file_stats_outputs = run_processor_test(gml_configs)?;

        // Assert the expected duplicate counts:
        assert_eq!(file_stats_outputs.get("file1.gml"), Some(&0));
        assert_eq!(file_stats_outputs.get("file2.gml"), Some(&0));
        assert_eq!(file_stats_outputs.get("file3.gml"), Some(&0));
        Ok(())
    }

    #[test]
    // An out-of-package member without gml:id is counted as well as reported.
    fn test_out_of_package_member_without_gml_id_is_counted() -> Result<(), BoxedError> {
        let temp_dir = TempDir::with_prefix("domain_of_definition_validator_test_")?;
        let codelists_dir = temp_dir.path().join("codelists");
        fs::create_dir_all(&codelists_dir)?;
        let file_path = temp_dir.path().join("file1.gml");
        fs::write(
            &file_path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0">
    <gml:boundedBy>
        <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
            <gml:lowerCorner>36.6 137.0 0</gml:lowerCorner>
            <gml:upperCorner>36.7 137.1 10</gml:upperCorner>
        </gml:Envelope>
    </gml:boundedBy>
    <core:cityObjectMember>
        <veg:PlantCover/>
    </core:cityObjectMember>
</core:CityModel>"#,
        )?;
        let mut feature = create_test_feature(
            "file1.gml",
            &format!("file://{}", file_path.display()),
            &codelists_dir,
        );
        let building = FeatureTypeDefinition {
            usable: true,
            paths: HashMap::new(),
        };
        let definitions = ObjectListDefinitions::new(HashMap::from([(
            "bldg".to_string(),
            HashMap::from([("bldg:Building".to_string(), building)]),
        )]));
        feature.insert(OBJECT_LIST_DEFINITIONS_ATTRIBUTE, definitions.into());

        let factory = DomainOfDefinitionValidatorFactory::new(&crate::plateau4::PLATEAU4);
        let mut processor: Box<dyn Processor> = factory.build(
            create_default_node_context(),
            EventHub::new(1024),
            "test".to_string(),
            None,
        )?;
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        processor.process(create_default_execute_context(feature), &fw)?;

        let ProcessorChannelForwarder::Noop(noop_fw) = &fw else {
            unreachable!();
        };
        let features = noop_fw.send_features.lock().unwrap();
        let flagged = |flag: &str| {
            features
                .iter()
                .filter(|feature| {
                    feature.get("flag") == Some(&AttributeValue::String(flag.to_string()))
                })
                .collect::<Vec<_>>()
        };
        let summary = flagged("Summary");
        assert_eq!(
            summary[0].get("invalidFeatureTypesNum"),
            Some(&AttributeValue::Number(Number::from(1)))
        );
        assert_eq!(flagged("ApplicationSchemaViolation").len(), 1);
        Ok(())
    }

    /// Processes one building file and returns every feature sent.
    fn process_building(
        lower_corner: &str,
        gml_id: &str,
        pos_list: &str,
    ) -> Result<Vec<Feature>, BoxedError> {
        let temp_dir = TempDir::with_prefix("domain_of_definition_validator_test_")?;
        let codelists_dir = temp_dir.path().join("codelists");
        fs::create_dir_all(&codelists_dir)?;
        let file_path = temp_dir.path().join("file1.gml");
        fs::write(
            &file_path,
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:bldg="http://www.opengis.net/citygml/building/2.0">
    <gml:boundedBy>
        <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
            <gml:lowerCorner>{lower_corner}</gml:lowerCorner>
            <gml:upperCorner>36.7 137.1 10</gml:upperCorner>
        </gml:Envelope>
    </gml:boundedBy>
    <core:cityObjectMember>
        <bldg:Building gml:id="{gml_id}">
            <bldg:lod0RoofEdge>
                <gml:MultiSurface>
                    <gml:surfaceMember>
                        <gml:Polygon>
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>{pos_list}</gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>
                </gml:MultiSurface>
            </bldg:lod0RoofEdge>
        </bldg:Building>
    </core:cityObjectMember>
</core:CityModel>"#
            ),
        )?;
        let feature = create_test_feature(
            "file1.gml",
            &format!("file://{}", file_path.display()),
            &codelists_dir,
        );

        let factory = DomainOfDefinitionValidatorFactory::new(&crate::plateau4::PLATEAU4);
        let mut processor: Box<dyn Processor> = factory.build(
            create_default_node_context(),
            EventHub::new(1024),
            "test".to_string(),
            None,
        )?;
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        processor.process(create_default_execute_context(feature), &fw)?;

        let ProcessorChannelForwarder::Noop(noop_fw) = &fw else {
            unreachable!();
        };
        let features = noop_fw.send_features.lock().unwrap().clone();
        Ok(features)
    }

    fn flagged<'a>(features: &'a [Feature], flag: &str) -> Vec<&'a Feature> {
        features
            .iter()
            .filter(|feature| {
                feature.get("flag") == Some(&AttributeValue::String(flag.to_string()))
            })
            .collect()
    }

    const VALID_POS_LIST: &str = "36.61 137.01 1 36.62 137.01 1 36.62 137.02 1 36.61 137.01 1";

    #[test]
    // A NaN envelope is one L06 error of the file with no geometry checked, and
    // the other checks still run.
    fn test_nan_envelope_fails_extents_check() -> Result<(), BoxedError> {
        let features = process_building("36.6 137.0 NaN", "bad_id", VALID_POS_LIST)?;

        assert!(flagged(&features, "ExtentsValidation").is_empty());

        let summary = flagged(&features, "Summary");
        assert_eq!(
            summary[0].get("incorrectExtents"),
            Some(&AttributeValue::Number(Number::from(1)))
        );
        assert_eq!(
            summary[0].get("gmlIdNotWellformed"),
            Some(&AttributeValue::Number(Number::from(1)))
        );
        assert_eq!(
            summary[0].get("srsName"),
            Some(&AttributeValue::String(VALID_SRS_NAME_6697.to_string()))
        );
        assert_eq!(
            summary[0].get("miscellaneous"),
            Some(&AttributeValue::String(
                CoordinateError::NonFiniteEnvelope.to_string()
            ))
        );
        Ok(())
    }

    #[test]
    // A geometry with a NaN coordinate fails L06.
    fn test_nan_coordinate_fails_extents_check() -> Result<(), BoxedError> {
        let features = process_building(
            "36.6 137.0 0",
            "bldg_bfbd43ad-101f-4b27-8978-3af8c136c24a",
            "36.61 137.01 1 36.62 137.01 NaN 36.62 137.02 1 36.61 137.01 1",
        )?;

        let errors = flagged(&features, "ExtentsValidation");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].get("minElevation"), Some(&AttributeValue::Null));
        assert_eq!(
            errors[0].get("minLatitude"),
            Some(&AttributeValue::Number(Number::from_f64(36.61).unwrap()))
        );

        assert_eq!(
            errors[0].get("miscellaneous"),
            Some(&AttributeValue::String(
                CoordinateError::NonFiniteGeometry.to_string()
            ))
        );

        let summary = flagged(&features, "Summary");
        assert_eq!(
            summary[0].get("incorrectExtents"),
            Some(&AttributeValue::Number(Number::from(1)))
        );
        assert_eq!(summary[0].get("miscellaneous"), None);
        Ok(())
    }

    /// Exceeds the envelope of [`city_model`] in elevation only.
    const OUTSIDE_POS_LIST: &str = "36.61 137.01 1 36.62 137.01 20 36.62 137.02 1 36.61 137.01 1";

    /// A gml:id of the form `{prefix}_{UUID}`, distinct for each `n`.
    fn gml_id(prefix: &str, n: u32) -> String {
        format!("{prefix}_00000000-0000-4000-8000-{n:012}")
    }

    /// A CityGML 3.0 city model whose envelope spans (36.6, 137.0, 0) to
    /// (36.7, 137.1, 10).
    fn city_model(srs_name: &str, members: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0" xmlns:gml="http://www.opengis.net/gml/3.2" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:bldg="http://www.opengis.net/citygml/building/3.0" xmlns:con="http://www.opengis.net/citygml/construction/3.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/3.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/3.0" xmlns:veg="http://www.opengis.net/citygml/vegetation/3.0" xmlns:urc="https://www.geospatial.jp/iur/urc/4.0" xmlns:foo="http://example.com/foo">
    <gml:boundedBy>
        <gml:Envelope srsName="{srs_name}" srsDimension="3">
            <gml:lowerCorner>36.6 137.0 0</gml:lowerCorner>
            <gml:upperCorner>36.7 137.1 10</gml:upperCorner>
        </gml:Envelope>
    </gml:boundedBy>
{members}
</core:CityModel>"#
        )
    }

    /// Writes, under `root/schemas`, the XSDs of the T03 (association target
    /// type) check: a surface member takes a surface, and an address is not
    /// one.
    fn write_schemas(root: &Path) -> std::io::Result<()> {
        write_file(
            &root.join("schemas/gml.xsd"),
            r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:gml="http://www.opengis.net/gml/3.2" targetNamespace="http://www.opengis.net/gml/3.2" elementFormDefault="qualified">
    <xs:element name="AbstractSurface" abstract="true"/>
    <xs:element name="Polygon" substitutionGroup="gml:AbstractSurface"/>
    <xs:complexType name="SurfacePropertyType">
        <xs:sequence minOccurs="0"><xs:element ref="gml:AbstractSurface"/></xs:sequence>
    </xs:complexType>
    <xs:element name="surfaceMember" type="gml:SurfacePropertyType"/>
</xs:schema>"#,
        )?;
        write_file(
            &root.join("schemas/core.xsd"),
            r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="http://www.opengis.net/citygml/3.0">
    <xs:element name="Address"/>
</xs:schema>"#,
        )
    }

    /// [`city_model`] naming the XSDs of [`write_schemas`], for a file two
    /// directories below `root`.
    fn city_model_with_schemas(srs_name: &str, members: &str) -> String {
        city_model(srs_name, members).replacen(
            r#"xmlns:foo="http://example.com/foo">"#,
            r#"xmlns:foo="http://example.com/foo" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.opengis.net/gml/3.2 ../../schemas/gml.xsd http://www.opengis.net/citygml/3.0 ../../schemas/core.xsd">"#,
            1,
        )
    }

    /// A building `gml:id` with an LoD0 surface of `pos_list`.
    fn building(gml_id: &str, pos_list: &str) -> String {
        format!(
            r#"    <core:cityObjectMember>
        <bldg:Building gml:id="{gml_id}">
            <core:lod0MultiSurface>
                <gml:MultiSurface>
                    <gml:surfaceMember>
                        <gml:Polygon>
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>{pos_list}</gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>
                </gml:MultiSurface>
            </core:lod0MultiSurface>
        </bldg:Building>
    </core:cityObjectMember>"#
        )
    }

    /// A city object group `gml_id` whose members are the xlink:hrefs `hrefs`,
    /// each in a role `{role_prefix}_{UUID}`.
    fn city_object_group(gml_id: &str, hrefs: &[String]) -> String {
        let members: String = hrefs
            .iter()
            .enumerate()
            .map(|(i, href)| {
                format!(
                    r#"
            <grp:groupMember>
                <grp:Role gml:id="{}">
                    <grp:groupMember xlink:href="{href}"/>
                </grp:Role>
            </grp:groupMember>"#,
                    gml_id.replacen("grp_", &format!("role{i}_"), 1)
                )
            })
            .collect();
        format!(
            r#"    <core:cityObjectMember>
        <grp:CityObjectGroup gml:id="{gml_id}">{members}
        </grp:CityObjectGroup>
    </core:cityObjectMember>"#
        )
    }

    /// The xpath of the xlink:hrefs of [`city_object_group`].
    const GROUP_MEMBER_XPATH: &str = "grp:CityObjectGroup/grp:groupMember/grp:Role/grp:groupMember";

    /// The DM geometric attribute of a city object, whose LoD0 geometry is
    /// `geometry`.
    fn dm_geometric_attribute(geometry: &str) -> String {
        format!(
            r#"
            <core:adeOfAbstractCityObject>
                <urc:DmGeometricAttribute>
                    <urc:dmCode>4265</urc:dmCode>
                    <urc:geometryType>E2</urc:geometryType>
                    <urc:mapLevel>2500</urc:mapLevel>
                    <urc:shapeType>0</urc:shapeType>
                    <urc:lod0Geometry>
                        {geometry}
                    </urc:lod0Geometry>
                </urc:DmGeometricAttribute>
            </core:adeOfAbstractCityObject>"#
        )
    }

    /// A codelist defining each of `codes`.
    fn codelist(codes: &[&str]) -> String {
        let definitions: String = codes
            .iter()
            .map(|code| {
                format!(
                    r#"
    <gml:dictionaryEntry>
        <gml:Definition gml:id="id{code}">
            <gml:description>{code}</gml:description>
            <gml:name>{code}</gml:name>
        </gml:Definition>
    </gml:dictionaryEntry>"#
                )
            })
            .collect();
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<gml:Dictionary xmlns:gml="http://www.opengis.net/gml" gml:id="codelist">
    <gml:name>codelist</gml:name>{definitions}
</gml:Dictionary>"#
        )
    }

    fn write_file(path: &Path, content: &str) -> std::io::Result<()> {
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, content)
    }

    fn file_uri(path: &Path) -> String {
        format!("file://{}", path.display())
    }

    /// An input feature for the city GML at `path`.
    fn input(
        name: &str,
        package: &str,
        index: u64,
        path: &Path,
        dir_codelists: Option<&Path>,
    ) -> Feature {
        let mut feature = Feature::new_with_attributes(Attributes::new());
        feature.insert("index", AttributeValue::Number(Number::from(index)));
        feature.insert("udxDirs", AttributeValue::String(package.to_string()));
        feature.insert("name", AttributeValue::String(name.to_string()));
        feature.insert("package", AttributeValue::String(package.to_string()));
        feature.insert("path", AttributeValue::String(file_uri(path)));
        if let Some(dir) = dir_codelists {
            feature.insert("dirCodelists", AttributeValue::String(file_uri(dir)));
        }
        feature
    }

    /// A sent feature as (port, attributes).
    type Output = (String, Value);

    /// Processes `inputs` in order with the PLATEAU6 profile, then finishes, and
    /// returns every feature sent.
    fn run(
        inputs: Vec<Feature>,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Vec<Output>, BoxedError> {
        run_with(inputs, with, |_| Ok(()))
    }

    /// [`run`], calling `before` with the index of each input before processing
    /// it.
    fn run_with(
        inputs: Vec<Feature>,
        with: Option<HashMap<String, Value>>,
        mut before: impl FnMut(usize) -> std::io::Result<()>,
    ) -> Result<Vec<Output>, BoxedError> {
        let factory = DomainOfDefinitionValidatorFactory::new(&crate::plateau6::PLATEAU6);
        let mut processor: Box<dyn Processor> = factory.build(
            create_default_node_context(),
            EventHub::new(1024),
            "test".to_string(),
            with,
        )?;
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        for (index, input) in inputs.into_iter().enumerate() {
            before(index)?;
            processor.process(create_default_execute_context(input), &fw)?;
        }
        processor.finish(create_default_node_context(), &fw)?;

        let ProcessorChannelForwarder::Noop(noop_fw) = &fw else {
            unreachable!();
        };
        let ports = noop_fw.send_ports.lock().unwrap();
        let features = noop_fw.send_features.lock().unwrap();
        Ok(ports
            .iter()
            .zip(features.iter())
            .map(|(port, feature)| {
                let attributes = AttributeValue::Map((*feature.attributes).clone());
                (port.as_ref().to_string(), Value::from(attributes))
            })
            .collect())
    }

    /// Orders the outputs of `finish`, which follow hash map order: the
    /// statistics by file name, and the duplicate gml:ids by gml:id, with
    /// their keys sorted and `duplicateId` renumbered in that order.
    fn normalize_finish(mut outputs: Vec<Output>) -> Vec<Output> {
        let start = outputs
            .iter()
            .position(|(port, feature)| {
                port == "duplicateGmlIdStats" || feature["flag"] == "GMLID_NotUnique"
            })
            .unwrap_or(outputs.len());
        let finish_outputs = outputs.split_off(start);
        // The statistics are sent before the duplicates.
        let stats_count = finish_outputs
            .iter()
            .take_while(|(port, _)| port == "duplicateGmlIdStats")
            .count();
        assert!(
            finish_outputs[stats_count..]
                .iter()
                .all(|(port, _)| port != "duplicateGmlIdStats"),
            "{finish_outputs:?}"
        );
        let (mut stats, mut duplicates): (Vec<_>, Vec<_>) = finish_outputs
            .into_iter()
            .partition(|(port, _)| port == "duplicateGmlIdStats");
        stats.sort_by(|a, b| a.1["filename"].as_str().cmp(&b.1["filename"].as_str()));
        duplicates.sort_by(|a, b| a.1["gmlId"].as_str().cmp(&b.1["gmlId"].as_str()));

        // One `duplicateId` per gml:id, distinct across gml:ids.
        let mut ids = Vec::<(Value, Value)>::new();
        for (_, feature) in &duplicates {
            assert_eq!(feature["flag"], "GMLID_NotUnique", "{feature}");
            assert!(feature["duplicateId"].is_string(), "{feature}");
            let id = (feature["gmlId"].clone(), feature["duplicateId"].clone());
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
        let gml_ids = ids
            .iter()
            .map(|(g, _)| g.to_string())
            .collect::<HashSet<_>>();
        let duplicate_ids = ids
            .iter()
            .map(|(_, d)| d.to_string())
            .collect::<HashSet<_>>();
        assert_eq!(gml_ids.len(), ids.len(), "{ids:?}");
        assert_eq!(duplicate_ids.len(), ids.len(), "{ids:?}");

        for (_, feature) in &mut duplicates {
            let rank = ids
                .iter()
                .position(|(g, _)| *g == feature["gmlId"])
                .unwrap();
            feature["duplicateId"] = Value::String(rank.to_string());
            let sorted = feature
                .as_object()
                .unwrap()
                .clone()
                .into_iter()
                .collect::<BTreeMap<_, _>>();
            *feature = Value::Object(sorted.into_iter().collect());
        }
        outputs.extend(stats);
        outputs.extend(duplicates);
        outputs
    }

    /// Compares the outputs, attribute order included.
    fn assert_outputs(actual: Vec<Output>, expected: Vec<Output>) {
        let pretty = |outputs| serde_json::to_string_pretty(&normalize_finish(outputs)).unwrap();
        assert_eq!(pretty(actual), pretty(expected));
    }

    /// `base` followed by the entries of `rest`; an entry of `rest` already in
    /// `base` replaces the value in place.
    fn concat(base: &Value, rest: Value) -> Value {
        let mut map = base.as_object().unwrap().clone();
        map.extend(rest.as_object().unwrap().clone());
        Value::Object(map)
    }

    fn feature_output(value: Value) -> Output {
        ("features".to_string(), value)
    }

    fn stats_output(filename: &str, count: u64) -> Output {
        (
            "duplicateGmlIdStats".to_string(),
            json!({"filename": filename, "duplicateGmlIdCount": count}),
        )
    }

    /// The attributes of the features sent with `flag`.
    fn with_flag<'a>(outputs: &'a [Output], flag: &str) -> Vec<&'a Value> {
        outputs
            .iter()
            .map(|(_, feature)| feature)
            .filter(|feature| feature["flag"] == flag)
            .collect()
    }

    /// The values of `keys` in each feature sent with `flag`.
    fn fields(outputs: &[Output], flag: &str, keys: &[&str]) -> Vec<Vec<Value>> {
        with_flag(outputs, flag)
            .into_iter()
            .map(|feature| keys.iter().map(|key| feature[*key].clone()).collect())
            .collect()
    }

    #[test]
    // Every check finds an error once, across two files and a city object group.
    fn test_all_outputs_in_send_order() -> Result<(), BoxedError> {
        let temp_dir = TempDir::with_prefix("domain_of_definition_validator_test_")?;
        let root = temp_dir.path();
        let codelists = root.join("codelists");
        write_file(&codelists.join("Building_class.xml"), &codelist(&["3001"]))?;
        write_file(&codelists.join("Building_usage.xml"), &codelist(&["411"]))?;

        let b1 = gml_id("bldg", 1);
        let b3 = gml_id("bldg", 3);
        let addr = gml_id("addr", 31);
        let poly = gml_id("poly", 32);
        let missing = gml_id("bldg", 99);
        let f4 = gml_id("frn", 4);
        let grp = gml_id("grp", 5);
        let missing_in_group = gml_id("bldg", 98);
        let f1 = root.join("udx/bldg/f1.gml");
        write_schemas(root)?;
        write_file(
            &f1,
            &city_model_with_schemas(
                VALID_SRS_NAME_6697,
                &format!(
                    r##"    <core:cityObjectMember>
        <bldg:Building gml:id="{b1}">
            <core:lod0MultiSurface>
                <gml:MultiSurface>
                    <gml:surfaceMember>
                        <gml:Polygon>
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>{VALID_POS_LIST}</gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>
                </gml:MultiSurface>
            </core:lod0MultiSurface>
            <bldg:class codeSpace="../../codelists/Building_class.xml">3001</bldg:class>
            <bldg:usage codeSpace="../../codelists/Building_usage.xml">9999</bldg:usage>
            <bldg:roofType codeSpace="../../codelists/Building_roofType.xml">1</bldg:roofType>
        </bldg:Building>
    </core:cityObjectMember>
{building_outside}
    <core:cityObjectMember>
        <bldg:Building gml:id="{b3}">
            <core:lod1Solid>
                <gml:Solid>
                    <gml:exterior>
                        <gml:Shell>
                            <gml:surfaceMember xlink:href="#{poly}"/>
                            <gml:surfaceMember xlink:href="#{missing}"/>
                            <gml:surfaceMember xlink:href="#{addr}"/>
                        </gml:Shell>
                    </gml:exterior>
                </gml:Solid>
            </core:lod1Solid>
            <core:lod2MultiSurface>
                <gml:MultiSurface>
                    <gml:surfaceMember>
                        <gml:Polygon gml:id="{poly}">
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>{VALID_POS_LIST}</gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>
                </gml:MultiSurface>
            </core:lod2MultiSurface>
            <bldg:address>
                <core:Address gml:id="{addr}"/>
            </bldg:address>
        </bldg:Building>
    </core:cityObjectMember>
    <core:cityObjectMember>
        <frn:CityFurniture gml:id="{f4}">{dm_solid}
        </frn:CityFurniture>
    </core:cityObjectMember>
{group}"##,
                    building_outside = building("bldg_not-a-uuid", OUTSIDE_POS_LIST),
                    dm_solid = dm_geometric_attribute("<gml:Solid/>"),
                    group = city_object_group(
                        &grp,
                        &[format!("#{poly}"), format!("#{missing_in_group}")]
                    ),
                ),
            ),
        )?;
        let f2 = root.join("udx/bldg/f2.gml");
        let srs_4326 = "http://www.opengis.net/def/crs/EPSG/0/4326";
        write_file(&f2, &city_model(srs_4326, &building(&b1, VALID_POS_LIST)))?;

        let outputs = run(
            vec![
                input("f1.gml", "bldg", 0, &f1, Some(&codelists)),
                input("f2.gml", "bldg", 1, &f2, Some(&codelists)),
            ],
            None,
        )?;

        let file1 = json!({"index": 0, "udxDirs": "bldg", "name": "f1.gml", "path": file_uri(&f1)});
        let member = |gml_id: &str, feature_type: &str| {
            concat(
                &file1,
                json!({"gmlId": gml_id, "featureType": feature_type}),
            )
        };
        let envelope = json!({
            "lowerLatitude": 36.6, "lowerLongitude": 137.0, "lowerElevation": 0.0,
            "upperLatitude": 36.7, "upperLongitude": 137.1, "upperElevation": 10.0,
        });
        let shell_surface_member =
            "bldg:Building/core:lod1Solid/gml:Solid/gml:exterior/gml:Shell/gml:surfaceMember";
        let expected = vec![
            feature_output(concat(
                &member(&b1, "Building"),
                json!({
                    "flag": "CodeValidation",
                    "tag": "bldg:usage",
                    "xpath": "bldg:Building/bldg:usage",
                    "codeSpace": "../../codelists/Building_usage.xml",
                    "codeSpaceValue": "9999",
                }),
            )),
            feature_output(concat(
                &member(&b1, "Building"),
                json!({
                    "flag": "incorrectCodeSpace",
                    "tag": "bldg:roofType",
                    "xpath": "bldg:Building/bldg:roofType",
                    "codeSpace": "../../codelists/Building_roofType.xml",
                    "codeSpaceValue": "1",
                }),
            )),
            feature_output(concat(
                &member("bldg_not-a-uuid", "Building"),
                json!({"flag": "GMLID_NotWellFormed", "tag": "bldg:Building"}),
            )),
            feature_output(concat(
                &concat(
                    &concat(
                        &member("bldg_not-a-uuid", "Building"),
                        json!({"flag": "ExtentsValidation", "isExtentsCorrect": false}),
                    ),
                    envelope.clone(),
                ),
                json!({
                    "minLatitude": 36.61, "minLongitude": 137.01, "minElevation": 1.0,
                    "maxLatitude": 36.62, "maxLongitude": 137.02, "maxElevation": 20.0,
                }),
            )),
            feature_output(concat(
                &member(&b3, "Building"),
                json!({
                    "flag": "XLink_NoReference",
                    "tag": "gml:surfaceMember",
                    "xpath": shell_surface_member,
                    "xlinkHref": format!("#{missing}"),
                }),
            )),
            feature_output(concat(
                &member(&b3, "Building"),
                json!({
                    "flag": "XLink_InvalidObjectType",
                    "tag": "gml:surfaceMember",
                    "xpath": shell_surface_member,
                    "xlinkHref": format!("#{addr}"),
                    "refGmlId": addr,
                    "refGmlTag": "core:Address",
                    "refGmlXpath": "bldg:Building/bldg:address/core:Address",
                }),
            )),
            feature_output(concat(
                &member(&f4, "CityFurniture"),
                json!({"flag": "GMLID_NotWellFormed", "tag": "frn:CityFurniture"}),
            )),
            // The DM geometric attribute has no gml:id.
            feature_output(concat(
                &member("", "CityFurniture"),
                json!({
                    "flag": "InvalidLodXGeometry",
                    "parentTag": "urc:DmGeometricAttribute",
                    "invalidGeometry": "gml:Solid",
                }),
            )),
            feature_output(json!({
                "index": 0,
                "udxDirs": "bldg",
                "name": "f1.gml",
                "flag": "XLink_NoReference",
                "tag": "grp:groupMember",
                "xpath": GROUP_MEMBER_XPATH,
                "xlinkHref": format!("#{missing_in_group}"),
                "featureType": "CityObjectGroup",
                "gmlId": grp,
            })),
            feature_output(concat(
                &concat(
                    &json!({
                        "index": 0,
                        "flag": "Summary",
                        "srsName": VALID_SRS_NAME_6697,
                        "invalidFeatureTypesNum": 1,
                        "invalidFeatureTypesDetail": {"CityFurniture": 1},
                        "correctCodeValues": 1,
                        "incorrectCodeValue": 1,
                        "incorrectCodeSpace": 1,
                        "isCorrectSrsName": true,
                        "correctExtents": 2,
                        "incorrectExtents": 1,
                    }),
                    envelope.clone(),
                ),
                json!({
                    "gmlIdNotWellformed": 2,
                    "xlinkHasNoReference": 2,
                    "xlinkInvalidObjectType": 1,
                    "invalidLodXGeometry": 1,
                }),
            )),
            feature_output(concat(
                &concat(
                    &json!({
                        "index": 1,
                        "flag": "Summary",
                        "srsName": srs_4326,
                        "invalidFeatureTypesNum": 0,
                        "invalidFeatureTypesDetail": {},
                        "correctCodeValues": 0,
                        "incorrectCodeValue": 0,
                        "incorrectCodeSpace": 0,
                        "isCorrectSrsName": false,
                        "correctExtents": 1,
                        "incorrectExtents": 0,
                    }),
                    envelope,
                ),
                json!({
                    "gmlIdNotWellformed": 0,
                    "xlinkHasNoReference": 0,
                    "xlinkInvalidObjectType": 0,
                    "invalidLodXGeometry": 0,
                }),
            )),
            stats_output("f1.gml", 1),
            stats_output("f2.gml", 1),
            feature_output(json!({
                "flag": "GMLID_NotUnique",
                "gmlId": b1,
                "duplicateId": "0",
                "numDuplicates": 2,
                "tag": "bldg:Building",
                "xpath": "bldg:Building",
                "index": "0",
                "udxDirs": "bldg",
                "name": "f1.gml",
            })),
            feature_output(json!({
                "flag": "GMLID_NotUnique",
                "gmlId": b1,
                "duplicateId": "0",
                "numDuplicates": 2,
                "tag": "bldg:Building",
                "xpath": "bldg:Building",
                "index": "1",
                "udxDirs": "bldg",
                "name": "f2.gml",
            })),
        ];
        assert_outputs(outputs, expected);
        Ok(())
    }

    #[test]
    // With an objectlist, the L03 errors of a file follow the errors of its
    // members, and precede the city object group xlink errors and the summary.
    fn test_all_outputs_with_objectlist_in_send_order() -> Result<(), BoxedError> {
        let temp_dir = TempDir::with_prefix("domain_of_definition_validator_test_")?;
        let root = temp_dir.path();
        let codelists = root.join("codelists");
        fs::create_dir_all(&codelists)?;
        let veg = gml_id("veg", 2);
        let grp = gml_id("grp", 3);
        let missing = gml_id("bldg", 99);
        let f1 = root.join("udx/bldg/f1.gml");
        write_file(
            &f1,
            &city_model(
                VALID_SRS_NAME_6697,
                &format!(
                    r#"    <core:cityObjectMember>
        <bldg:Building gml:id="bldg_not-a-uuid"/>
    </core:cityObjectMember>
    <core:cityObjectMember>
        <veg:PlantCover gml:id="{veg}"/>
    </core:cityObjectMember>
{group}"#,
                    group = city_object_group(&grp, &[format!("#{missing}")]),
                ),
            ),
        )?;
        let mut feature = input("f1.gml", "bldg", 0, &f1, Some(&codelists));
        let building = FeatureTypeDefinition {
            usable: true,
            paths: HashMap::new(),
        };
        let definitions = ObjectListDefinitions::new(HashMap::from([(
            "bldg".to_string(),
            HashMap::from([("bldg:Building".to_string(), building)]),
        )]));
        feature.insert(OBJECT_LIST_DEFINITIONS_ATTRIBUTE, definitions.into());

        let outputs = run(vec![feature], None)?;

        let file1 = json!({"index": 0, "udxDirs": "bldg", "name": "f1.gml"});
        let file1_path = concat(&file1, json!({"path": file_uri(&f1)}));
        let expected = vec![
            feature_output(concat(
                &file1_path,
                json!({
                    "gmlId": "bldg_not-a-uuid",
                    "featureType": "Building",
                    "flag": "GMLID_NotWellFormed",
                    "tag": "bldg:Building",
                }),
            )),
            feature_output(concat(
                &file1_path,
                json!({
                    "gmlId": veg,
                    "featureType": "PlantCover",
                    "flag": "GMLID_NotWellFormed",
                    "tag": "veg:PlantCover",
                }),
            )),
            feature_output(concat(
                &file1,
                json!({
                    "flag": "ApplicationSchemaViolation",
                    "featureType": "veg:PlantCover",
                    "gmlId": veg,
                    "path": "",
                    "reason": Reason::Undefined.as_str(),
                }),
            )),
            feature_output(concat(
                &file1,
                json!({
                    "flag": "XLink_NoReference",
                    "tag": "grp:groupMember",
                    "xpath": GROUP_MEMBER_XPATH,
                    "xlinkHref": format!("#{missing}"),
                    "featureType": "CityObjectGroup",
                    "gmlId": grp,
                }),
            )),
            feature_output(json!({
                "index": 0,
                "flag": "Summary",
                "srsName": VALID_SRS_NAME_6697,
                "invalidFeatureTypesNum": 1,
                "invalidFeatureTypesDetail": {},
                "correctCodeValues": 0,
                "incorrectCodeValue": 0,
                "incorrectCodeSpace": 0,
                "isCorrectSrsName": true,
                "correctExtents": 0,
                "incorrectExtents": 0,
                "lowerLatitude": 36.6, "lowerLongitude": 137.0, "lowerElevation": 0.0,
                "upperLatitude": 36.7, "upperLongitude": 137.1, "upperElevation": 10.0,
                "gmlIdNotWellformed": 2,
                "xlinkHasNoReference": 1,
                "xlinkInvalidObjectType": 0,
                "invalidLodXGeometry": 0,
            })),
            stats_output("f1.gml", 0),
        ];
        assert_outputs(outputs, expected);
        Ok(())
    }

    #[test]
    // Each of the five ways to find the codelist of a codeSpace, and a codeSpace
    // found in none of them.
    fn test_code_value_codelist_resolution() -> Result<(), BoxedError> {
        let temp_dir = TempDir::with_prefix("domain_of_definition_validator_test_")?;
        let root = temp_dir.path();
        let codelists = root.join("codelists");
        for name in ["A", "B", "D", "E"] {
            write_file(&codelists.join(format!("{name}.xml")), &codelist(&["1"]))?;
        }
        let other_codelists = root.join("other");
        write_file(&other_codelists.join("C.xml"), &codelist(&["1"]))?;
        let fallback = root.join("fallback");
        write_file(&fallback.join("F.xml"), &codelist(&["1"]))?;
        let no_fallback = root.join("none");

        // (file, codeSpace, dirCodelists, codelistsPath). A codeSpace starting
        // with `../` does not lead to the codelist from the city GML.
        let cases = [
            // In the preloaded codelists, by the part after `codelists/`.
            ("a", "../codelists/A.xml", Some(&codelists), &no_fallback),
            // In the preloaded codelists, by joining the part after
            // `codelists/` to dirCodelists.
            (
                "b",
                "../codelists/sub/../B.xml",
                Some(&codelists),
                &no_fallback,
            ),
            // Loaded from dirCodelists, which differs from the one preloaded.
            (
                "c",
                "../codelists/C.xml",
                Some(&other_codelists),
                &no_fallback,
            ),
            // In the preloaded codelists, relative to the city GML.
            ("d", "../../codelists/D.xml", None, &no_fallback),
            // In the preloaded codelists, by the file name in codelistsPath.
            ("e", "../../elsewhere/E.xml", Some(&codelists), &codelists),
            // Loaded by the file name from codelistsPath.
            ("f", "../../elsewhere/F.xml", Some(&codelists), &fallback),
            // Nowhere.
            ("g", "../../codelists/G.xml", Some(&codelists), &codelists),
        ];
        let mut inputs = Vec::new();
        for (index, (name, code_space, dir_codelists, codelists_path)) in cases.iter().enumerate() {
            let path = root.join(format!("udx/bldg/{name}.gml"));
            write_file(
                &path,
                &city_model(
                    VALID_SRS_NAME_6697,
                    &format!(
                        r#"    <core:cityObjectMember>
        <bldg:Building gml:id="{}">
            <bldg:class codeSpace="{code_space}">1</bldg:class>
            <bldg:usage codeSpace="{code_space}">2</bldg:usage>
        </bldg:Building>
    </core:cityObjectMember>"#,
                        gml_id("bldg", index as u32)
                    ),
                ),
            )?;
            let mut feature = input(
                &format!("{name}.gml"),
                "bldg",
                index as u64,
                &path,
                dir_codelists.map(|dir| dir.as_path()),
            );
            feature.insert(
                "codelistsPath",
                AttributeValue::String(file_uri(codelists_path)),
            );
            inputs.push(feature);
        }
        let with = HashMap::from([(
            "codelistsPath".to_string(),
            json!({"type": "flowExpr", "value": r#"attributes["codelistsPath"]"#}),
        )]);

        let outputs = run(inputs, Some(with))?;

        // (index, correctCodeValues, incorrectCodeValue, incorrectCodeSpace)
        let counts = fields(
            &outputs,
            "Summary",
            &[
                "index",
                "correctCodeValues",
                "incorrectCodeValue",
                "incorrectCodeSpace",
            ],
        );
        assert_eq!(
            counts,
            [
                [0, 1, 1, 0],
                [1, 1, 1, 0],
                [2, 1, 1, 0],
                [3, 1, 1, 0],
                [4, 1, 1, 0],
                [5, 1, 1, 0],
                [6, 0, 0, 2],
            ]
            .map(|row| row.map(|v| json!(v)).to_vec())
            .to_vec()
        );
        assert_eq!(
            fields(&outputs, "CodeValidation", &["name", "codeSpaceValue"]).len(),
            6
        );
        Ok(())
    }

    #[test]
    // L05 accepts EPSG:6697 and EPSG:6668, and the CRSs of unf for unf only. A
    // file that cannot be parsed is not an L05 error.
    fn test_srs_name() -> Result<(), BoxedError> {
        let temp_dir = TempDir::with_prefix("domain_of_definition_validator_test_")?;
        let root = temp_dir.path();
        let codelists = root.join("codelists");
        fs::create_dir_all(&codelists)?;
        let srs_10169 = "http://www.opengis.net/def/crs/EPSG/0/10169";
        let srs_4326 = "http://www.opengis.net/def/crs/EPSG/0/4326";
        let not_well_formed = r#"<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0" xmlns:gml="http://www.opengis.net/gml/3.2">
    <gml:boundedBy>
</core:CityModel>"#;
        let no_envelope = r#"<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0" xmlns:gml="http://www.opengis.net/gml/3.2">
</core:CityModel>"#;
        let cases = [
            ("bldg", city_model(VALID_SRS_NAME_6697, "")),
            ("bldg", city_model(VALID_SRS_NAME_6668, "")),
            ("bldg", city_model(srs_4326, "")),
            ("unf", city_model(srs_10169, "")),
            ("bldg", city_model(srs_10169, "")),
            ("bldg", not_well_formed.to_string()),
            ("bldg", no_envelope.to_string()),
        ];
        let mut inputs = Vec::new();
        for (index, (package, content)) in cases.iter().enumerate() {
            let path = root.join(format!("udx/{package}/{index}.gml"));
            write_file(&path, content)?;
            inputs.push(input(
                &format!("{index}.gml"),
                package,
                index as u64,
                &path,
                Some(&codelists),
            ));
        }

        let outputs = run(inputs, None)?;

        assert_eq!(
            fields(&outputs, "Summary", &["srsName", "isCorrectSrsName"]),
            [
                (VALID_SRS_NAME_6697, true),
                (VALID_SRS_NAME_6668, true),
                (srs_4326, false),
                (srs_10169, true),
                (srs_10169, false),
                ("", true),
                ("", true),
            ]
            .map(|(srs_name, correct)| vec![json!(srs_name), json!(correct)])
            .to_vec()
        );
        Ok(())
    }

    #[test]
    // The gml:id of a member has the package prefix, and that of a city object
    // group any lowercase prefix. The gml:ids within a member are not checked.
    fn test_gml_id_format() -> Result<(), BoxedError> {
        let temp_dir = TempDir::with_prefix("domain_of_definition_validator_test_")?;
        let root = temp_dir.path();
        let codelists = root.join("codelists");
        fs::create_dir_all(&codelists)?;
        let upper_case = "bldg_ABCDEF00-0000-4000-8000-000000000003".to_string();
        let members = [
            ("bldg:Building", gml_id("bldg", 1)),
            ("bldg:Building", gml_id("frn", 2)),
            ("bldg:Building", upper_case.clone()),
            ("bldg:Building", "bldg_not-a-uuid".to_string()),
            ("grp:CityObjectGroup", gml_id("grp", 5)),
            ("grp:CityObjectGroup", gml_id("abc", 6)),
            ("grp:CityObjectGroup", "grp_not-a-uuid".to_string()),
        ]
        .map(|(tag, id)| {
            format!(
                r#"    <core:cityObjectMember>
        <{tag} gml:id="{id}">
            <core:lod0MultiSurface>
                <gml:MultiSurface gml:id="not-a-uuid"/>
            </core:lod0MultiSurface>
        </{tag}>
    </core:cityObjectMember>"#
            )
        })
        .join("\n");
        let f1 = root.join("udx/bldg/f1.gml");
        write_file(&f1, &city_model(VALID_SRS_NAME_6697, &members))?;

        let outputs = run(
            vec![input("f1.gml", "bldg", 0, &f1, Some(&codelists))],
            None,
        )?;

        assert_eq!(
            fields(&outputs, "GMLID_NotWellFormed", &["gmlId", "tag"]),
            [
                (gml_id("frn", 2), "bldg:Building"),
                (upper_case, "bldg:Building"),
                ("bldg_not-a-uuid".to_string(), "bldg:Building"),
                ("grp_not-a-uuid".to_string(), "grp:CityObjectGroup"),
            ]
            .map(|(id, tag)| vec![json!(id), json!(tag)])
            .to_vec()
        );
        assert_eq!(
            fields(&outputs, "Summary", &["gmlIdNotWellformed"]),
            [[json!(4)]]
        );
        Ok(())
    }

    #[test]
    // A gml:id is counted once per member, by its last occurrence there, and
    // the duplicates are found within a file as well as across files.
    fn test_gml_id_uniqueness() -> Result<(), BoxedError> {
        let temp_dir = TempDir::with_prefix("domain_of_definition_validator_test_")?;
        let root = temp_dir.path();
        let codelists = root.join("codelists");
        fs::create_dir_all(&codelists)?;
        let b1 = gml_id("bldg", 1);
        let b3 = gml_id("bldg", 3);
        let b4 = gml_id("bldg", 4);
        let b5 = gml_id("bldg", 5);
        let poly = gml_id("poly", 6);
        let polygon = |gml_id: &str| {
            format!(
                r#"<gml:MultiSurface>
                    <gml:surfaceMember>
                        <gml:Polygon gml:id="{gml_id}"/>
                    </gml:surfaceMember>
                </gml:MultiSurface>"#
            )
        };
        let f1 = root.join("udx/bldg/f1.gml");
        write_file(
            &f1,
            &city_model(
                VALID_SRS_NAME_6697,
                &format!(
                    r#"    <core:cityObjectMember>
        <bldg:Building gml:id="{b1}"/>
    </core:cityObjectMember>
    <core:cityObjectMember>
        <bldg:Building gml:id="{b1}"/>
    </core:cityObjectMember>
    <core:cityObjectMember>
        <bldg:Building gml:id="{b3}">
            <core:lod2MultiSurface>
                {poly_in_lod2}
            </core:lod2MultiSurface>
            <core:lod3MultiSurface>
                {poly_in_lod3}
            </core:lod3MultiSurface>
        </bldg:Building>
    </core:cityObjectMember>
    <core:cityObjectMember>
        <bldg:Building gml:id="{b4}">
            <core:lod2MultiSurface>
                {poly_in_lod2}
            </core:lod2MultiSurface>
        </bldg:Building>
    </core:cityObjectMember>
    <core:cityObjectMember>
        <bldg:Building gml:id="{b5}">
            <core:lod2MultiSurface>
                {b5_in_lod2}
            </core:lod2MultiSurface>
        </bldg:Building>
    </core:cityObjectMember>"#,
                    poly_in_lod2 = polygon(&poly),
                    poly_in_lod3 = polygon(&poly),
                    b5_in_lod2 = polygon(&b5),
                ),
            ),
        )?;
        let f2 = root.join("udx/bldg/f2.gml");
        write_file(
            &f2,
            &city_model(
                VALID_SRS_NAME_6697,
                &format!(
                    r#"    <core:cityObjectMember>
        <bldg:Building gml:id="{b5}"/>
    </core:cityObjectMember>
    <core:cityObjectMember>
        <bldg:Building gml:id="{}"/>
    </core:cityObjectMember>"#,
                    gml_id("bldg", 21)
                ),
            ),
        )?;
        let f3 = root.join("udx/bldg/f3.gml");
        write_file(
            &f3,
            &city_model(VALID_SRS_NAME_6697, &building(&gml_id("bldg", 31), "")),
        )?;

        let outputs = run(
            vec![
                input("f1.gml", "bldg", 0, &f1, Some(&codelists)),
                input("f2.gml", "bldg", 1, &f2, Some(&codelists)),
                input("f3.gml", "bldg", 2, &f3, Some(&codelists)),
            ],
            None,
        )?;

        let start = outputs
            .iter()
            .position(|(port, _)| port == "duplicateGmlIdStats")
            .unwrap();
        let lod2_polygon =
            "bldg:Building/core:lod2MultiSurface/gml:MultiSurface/gml:surfaceMember/gml:Polygon";
        let lod3_polygon =
            "bldg:Building/core:lod3MultiSurface/gml:MultiSurface/gml:surfaceMember/gml:Polygon";
        let duplicate =
            |gml_id: &str, id: &str, num: u64, tag: &str, xpath: &str, file: (u64, &str)| {
                feature_output(json!({
                    "flag": "GMLID_NotUnique",
                    "gmlId": gml_id,
                    "duplicateId": id,
                    "numDuplicates": num,
                    "tag": tag,
                    "xpath": xpath,
                    "index": file.0.to_string(),
                    "udxDirs": "bldg",
                    "name": file.1,
                }))
            };
        assert_outputs(
            outputs[start..].to_vec(),
            vec![
                stats_output("f1.gml", 5),
                stats_output("f2.gml", 1),
                stats_output("f3.gml", 0),
                duplicate(&b1, "0", 2, "bldg:Building", "bldg:Building", (0, "f1.gml")),
                duplicate(&b1, "0", 2, "bldg:Building", "bldg:Building", (0, "f1.gml")),
                // The member's own gml:id is replaced by the polygon's.
                duplicate(&b5, "1", 2, "gml:Polygon", lod2_polygon, (0, "f1.gml")),
                duplicate(&b5, "1", 2, "bldg:Building", "bldg:Building", (1, "f2.gml")),
                // Only the last of the two in the same member.
                duplicate(&poly, "2", 2, "gml:Polygon", lod3_polygon, (0, "f1.gml")),
                duplicate(&poly, "2", 2, "gml:Polygon", lod2_polygon, (0, "f1.gml")),
            ],
        );
        Ok(())
    }

    #[test]
    // A member's xlink refers to a gml:id within the member, and that of a city
    // object group to one anywhere in the file.
    fn test_xlink_reference() -> Result<(), BoxedError> {
        let temp_dir = TempDir::with_prefix("domain_of_definition_validator_test_")?;
        let root = temp_dir.path();
        let codelists = root.join("codelists");
        fs::create_dir_all(&codelists)?;

        let grp = gml_id("grp", 1);
        let b2 = gml_id("bldg", 2);
        let b3 = gml_id("bldg", 3);
        let addr2 = gml_id("addr", 21);
        let poly2 = gml_id("poly", 22);
        let poly3 = gml_id("poly", 31);
        let missing = gml_id("bldg", 99);
        let f1 = root.join("udx/bldg/f1.gml");
        write_schemas(root)?;
        write_file(
            &f1,
            &city_model_with_schemas(
                VALID_SRS_NAME_6697,
                &format!(
                    r##"{group}
    <core:cityObjectMember>
        <bldg:Building gml:id="{b2}">
            <core:lod1Solid>
                <gml:Solid>
                    <gml:exterior>
                        <gml:Shell>
                            <gml:surfaceMember xlink:href="#{poly2}"/>
                            <gml:surfaceMember xlink:href="#{addr2}"/>
                            <gml:surfaceMember xlink:href="#{poly3}"/>
                            <gml:surfaceMember xlink:href="#{missing}"/>
                        </gml:Shell>
                    </gml:exterior>
                </gml:Solid>
            </core:lod1Solid>
            <core:lod2MultiSurface>
                <gml:MultiSurface>
                    <gml:surfaceMember>
                        <gml:Polygon gml:id="{poly2}"/>
                    </gml:surfaceMember>
                </gml:MultiSurface>
            </core:lod2MultiSurface>
            <bldg:address>
                <core:Address gml:id="{addr2}"/>
            </bldg:address>
        </bldg:Building>
    </core:cityObjectMember>
    <core:cityObjectMember>
        <bldg:Building gml:id="{b3}">
            <core:lod2MultiSurface>
                <gml:MultiSurface>
                    <gml:surfaceMember>
                        <gml:Polygon gml:id="{poly3}"/>
                    </gml:surfaceMember>
                </gml:MultiSurface>
            </core:lod2MultiSurface>
        </bldg:Building>
    </core:cityObjectMember>"##,
                    group = city_object_group(
                        &grp,
                        &[format!("#{b3}"), format!("#{poly3}"), format!("#{missing}"),]
                    ),
                ),
            ),
        )?;

        let outputs = run(
            vec![input("f1.gml", "bldg", 0, &f1, Some(&codelists))],
            None,
        )?;

        let keys = ["flag", "gmlId", "xlinkHref"];
        let xlink_errors = fields(&outputs, "XLink_InvalidObjectType", &keys)
            .into_iter()
            .chain(fields(&outputs, "XLink_NoReference", &keys))
            .collect::<Vec<_>>();
        assert_eq!(
            xlink_errors,
            [
                ("XLink_InvalidObjectType", &b2, format!("#{addr2}")),
                // Another member's gml:id.
                ("XLink_NoReference", &b2, format!("#{poly3}")),
                ("XLink_NoReference", &b2, format!("#{missing}")),
                ("XLink_NoReference", &grp, format!("#{missing}")),
            ]
            .map(|(flag, id, href)| vec![json!(flag), json!(id), json!(href)])
            .to_vec()
        );
        assert_eq!(
            fields(
                &outputs,
                "Summary",
                &["xlinkHasNoReference", "xlinkInvalidObjectType"]
            ),
            [[json!(3), json!(1)]]
        );
        Ok(())
    }

    /// The gml:ids of [`process_external_references`].
    struct ExternalReferenceIds {
        building: String,
        not_well_formed: String,
        group: String,
        /// In the external GML file.
        external_polygon: String,
        /// In no file.
        missing: String,
        /// The external GML file as an xlink names it.
        external: String,
        /// A GML file that does not exist, as an xlink names it.
        nonexistent: String,
    }

    /// Checks a file whose member refers to gml:ids of another GML file, each
    /// reference written as `{href_path(file)}#{gml:id}`, and returns every
    /// feature sent.
    fn process_external_references(
        href_path: impl Fn(&Path) -> String,
    ) -> Result<(Vec<Output>, ExternalReferenceIds), BoxedError> {
        let temp_dir = TempDir::with_prefix("domain_of_definition_validator_test_")?;
        let root = temp_dir.path();
        let codelists = root.join("codelists");
        fs::create_dir_all(&codelists)?;

        let ids = ExternalReferenceIds {
            building: gml_id("bldg", 1),
            not_well_formed: "bldg_not-a-uuid".to_string(),
            group: gml_id("grp", 3),
            external_polygon: gml_id("poly", 91),
            missing: gml_id("bldg", 99),
            external: href_path(&root.join("udx/bldg/external.gml")),
            nonexistent: href_path(&root.join("udx/bldg/nonexistent.gml")),
        };
        let ExternalReferenceIds {
            building,
            not_well_formed,
            group,
            external_polygon,
            missing,
            external,
            nonexistent,
        } = &ids;

        // The polygon is nested in a member, not the root element.
        write_file(
            &root.join("udx/bldg/external.gml"),
            &city_model(
                VALID_SRS_NAME_6697,
                &format!(
                    r#"    <core:cityObjectMember>
        <bldg:Building gml:id="{}">
            <core:lod2MultiSurface>
                <gml:MultiSurface>
                    <gml:surfaceMember>
                        <gml:Polygon gml:id="{external_polygon}"/>
                    </gml:surfaceMember>
                </gml:MultiSurface>
            </core:lod2MultiSurface>
        </bldg:Building>
    </core:cityObjectMember>"#,
                    gml_id("bldg", 92)
                ),
            ),
        )?;

        let f1 = root.join("udx/bldg/f1.gml");
        write_file(
            &f1,
            &city_model(
                VALID_SRS_NAME_6697,
                &format!(
                    r##"    <core:cityObjectMember>
        <bldg:Building gml:id="{building}">
            <core:lod1Solid>
                <gml:Solid>
                    <gml:exterior>
                        <gml:Shell>
                            <gml:surfaceMember xlink:href="{external}#{external_polygon}"/>
                            <gml:surfaceMember xlink:href="{external}#{missing}"/>
                            <gml:surfaceMember xlink:href="{nonexistent}#{external_polygon}"/>
                            <gml:surfaceMember xlink:href="#{missing}"/>
                        </gml:Shell>
                    </gml:exterior>
                </gml:Solid>
            </core:lod1Solid>
        </bldg:Building>
    </core:cityObjectMember>
    <core:cityObjectMember>
        <bldg:Building gml:id="{not_well_formed}"/>
    </core:cityObjectMember>
{group_member}"##,
                    group_member = city_object_group(group, &[format!("#{missing}")]),
                ),
            ),
        )?;

        let outputs = run(
            vec![input("f1.gml", "bldg", 0, &f1, Some(&codelists))],
            None,
        )?;
        Ok((outputs, ids))
    }

    /// Asserts the errors of [`process_external_references`]: a reference to a
    /// gml:id in the external file is valid when the file `is_read`, one to a
    /// gml:id or a file that does not exist has no referent, and the other
    /// xlinks and members of the file are still checked.
    fn assert_external_references(outputs: &[Output], ids: &ExternalReferenceIds, is_read: bool) {
        let ExternalReferenceIds {
            building,
            not_well_formed,
            group,
            external_polygon,
            missing,
            external,
            nonexistent,
        } = ids;
        let found_in_external =
            (!is_read).then(|| (building, format!("{external}#{external_polygon}")));
        let no_references = found_in_external
            .into_iter()
            .chain([
                (building, format!("{external}#{missing}")),
                (building, format!("{nonexistent}#{external_polygon}")),
                (building, format!("#{missing}")),
                (group, format!("#{missing}")),
            ])
            .map(|(id, href)| vec![json!(id), json!(href)])
            .collect::<Vec<_>>();
        assert_eq!(
            fields(outputs, "XLink_NoReference", &["gmlId", "xlinkHref"]),
            no_references
        );
        assert!(with_flag(outputs, "XLink_InvalidObjectType").is_empty());
        assert_eq!(
            fields(outputs, "GMLID_NotWellFormed", &["gmlId"]),
            [[json!(not_well_formed)]]
        );
        assert_eq!(
            fields(
                outputs,
                "Summary",
                &[
                    "isCorrectSrsName",
                    "gmlIdNotWellformed",
                    "xlinkHasNoReference",
                    "xlinkInvalidObjectType",
                ]
            ),
            [[json!(true), json!(1), json!(no_references.len()), json!(0)]]
        );
    }

    #[test]
    // An xlink to `{absolute URI of a GML file}#{gml:id}` has no referent, as
    // the file is not read.
    fn test_xlink_to_external_file_by_uri() -> Result<(), BoxedError> {
        let (outputs, ids) = process_external_references(file_uri)?;

        assert_external_references(&outputs, &ids, false);
        Ok(())
    }

    #[test]
    // An xlink to `{GML file in the directory of the referring one}#{gml:id}`
    // refers to a gml:id anywhere in that file.
    fn test_xlink_to_external_file_by_relative_path() -> Result<(), BoxedError> {
        let relative = |path: &Path| path.file_name().unwrap().to_string_lossy().to_string();
        let (outputs, ids) = process_external_references(relative)?;

        assert_external_references(&outputs, &ids, true);
        Ok(())
    }

    #[test]
    // An xlink refers to a GML file in the directory of the referring one, by
    // its file name with or without `./`, and to no other file.
    fn test_xlink_to_external_file_only_in_same_directory() -> Result<(), BoxedError> {
        let temp_dir = TempDir::with_prefix("domain_of_definition_validator_test_")?;
        let root = temp_dir.path();
        let codelists = root.join("codelists");
        fs::create_dir_all(&codelists)?;
        let polygon = gml_id("poly", 91);
        let external = external_city_model(&polygon);
        write_file(&root.join("udx/bldg/external.gml"), &external)?;
        write_file(&root.join("udx/bldg/sub/external.gml"), &external)?;
        let hrefs = [
            "external.gml",
            "./external.gml",
            "sub/external.gml",
            "../bldg/external.gml",
        ]
        .map(|path| format!("{path}#{polygon}"));
        let members = hrefs
            .iter()
            .enumerate()
            .map(|(i, href)| {
                building_with_reference(&gml_id("bldg", i as u32), "../../codelists/C.xml", href)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let f1 = root.join("udx/bldg/f1.gml");
        write_file(&f1, &city_model(VALID_SRS_NAME_6697, &members))?;

        let outputs = run(
            vec![input("f1.gml", "bldg", 0, &f1, Some(&codelists))],
            None,
        )?;

        assert_eq!(
            fields(&outputs, "XLink_NoReference", &["xlinkHref"]),
            [[json!(hrefs[2])], [json!(hrefs[3])]]
        );
        Ok(())
    }

    /// A building `gml_id` with a code value `1` of `code_space` and a surface
    /// referring to `href`.
    fn building_with_reference(gml_id: &str, code_space: &str, href: &str) -> String {
        format!(
            r#"    <core:cityObjectMember>
        <bldg:Building gml:id="{gml_id}">
            <core:lod1Solid>
                <gml:Solid>
                    <gml:exterior>
                        <gml:Shell>
                            <gml:surfaceMember xlink:href="{href}"/>
                        </gml:Shell>
                    </gml:exterior>
                </gml:Solid>
            </core:lod1Solid>
            <bldg:class codeSpace="{code_space}">1</bldg:class>
        </bldg:Building>
    </core:cityObjectMember>"#
        )
    }

    /// A city model with a building that has the polygon `polygon_id`.
    fn external_city_model(polygon_id: &str) -> String {
        city_model(
            VALID_SRS_NAME_6697,
            &format!(
                r#"    <core:cityObjectMember>
        <bldg:Building gml:id="{}">
            <core:lod2MultiSurface>
                <gml:MultiSurface>
                    <gml:surfaceMember>
                        <gml:Polygon gml:id="{polygon_id}"/>
                    </gml:surfaceMember>
                </gml:MultiSurface>
            </core:lod2MultiSurface>
        </bldg:Building>
    </core:cityObjectMember>"#,
                gml_id("bldg", 92)
            ),
        )
    }

    #[test]
    // An external GML file that ends with an element open is not read.
    fn test_xlink_to_truncated_external_file() -> Result<(), BoxedError> {
        let temp_dir = TempDir::with_prefix("domain_of_definition_validator_test_")?;
        let root = temp_dir.path();
        let codelists = root.join("codelists");
        fs::create_dir_all(&codelists)?;
        let polygon = gml_id("poly", 91);
        let complete = external_city_model(&polygon);
        let truncated = &complete[..complete.rfind("</core:CityModel>").unwrap()];
        write_file(&root.join("udx/bldg/external.gml"), truncated)?;
        let building = gml_id("bldg", 1);
        let href = format!("external.gml#{polygon}");
        let f1 = root.join("udx/bldg/f1.gml");
        write_file(
            &f1,
            &city_model(
                VALID_SRS_NAME_6697,
                &building_with_reference(&building, "../../codelists/C.xml", &href),
            ),
        )?;

        let outputs = run(
            vec![input("f1.gml", "bldg", 0, &f1, Some(&codelists))],
            None,
        )?;

        assert_eq!(
            fields(&outputs, "XLink_NoReference", &["gmlId", "xlinkHref"]),
            [[json!(building), json!(href)]]
        );
        Ok(())
    }

    #[test]
    // A codelist or an external GML file that cannot be read for a file is
    // read again for a later file.
    fn test_unreadable_files_are_read_again_by_later_files() -> Result<(), BoxedError> {
        let temp_dir = TempDir::with_prefix("domain_of_definition_validator_test_")?;
        let root = temp_dir.path();
        let codelists = root.join("codelists");
        fs::create_dir_all(&codelists)?;
        let fallback = root.join("fallback");
        let polygon = gml_id("poly", 91);
        let href = format!("external.gml#{polygon}");
        let mut inputs = Vec::new();
        for index in 0..2 {
            let path = root.join(format!("udx/bldg/f{index}.gml"));
            write_file(
                &path,
                &city_model(
                    VALID_SRS_NAME_6697,
                    &building_with_reference(
                        &gml_id("bldg", index),
                        "../../elsewhere/F.xml",
                        &href,
                    ),
                ),
            )?;
            inputs.push(input(
                &format!("f{index}.gml"),
                "bldg",
                index as u64,
                &path,
                Some(&codelists),
            ));
        }
        let with = HashMap::from([(
            "codelistsPath".to_string(),
            json!({"type": "string", "value": file_uri(&fallback)}),
        )]);

        // Both files appear only after the first city GML is checked.
        let outputs = run_with(inputs, Some(with), |index| {
            if index == 1 {
                write_file(&fallback.join("F.xml"), &codelist(&["1"]))?;
                write_file(
                    &root.join("udx/bldg/external.gml"),
                    &external_city_model(&polygon),
                )?;
            }
            Ok(())
        })?;

        assert_eq!(
            fields(
                &outputs,
                "Summary",
                &[
                    "incorrectCodeSpace",
                    "correctCodeValues",
                    "xlinkHasNoReference"
                ]
            ),
            [
                [json!(1), json!(0), json!(1)],
                [json!(0), json!(1), json!(0)]
            ]
        );
        Ok(())
    }

    #[test]
    // The geometry types allowed in the LoD0 geometry of a DM geometric
    // attribute.
    fn test_lod_geometry_type() -> Result<(), BoxedError> {
        let temp_dir = TempDir::with_prefix("domain_of_definition_validator_test_")?;
        let root = temp_dir.path();
        let codelists = root.join("codelists");
        fs::create_dir_all(&codelists)?;
        let frn1 = gml_id("frn", 1);
        let veg2 = gml_id("veg", 2);
        let dm = |geometries: &[&str]| -> String {
            geometries
                .iter()
                .map(|geometry| dm_geometric_attribute(geometry))
                .collect()
        };
        let f1 = root.join("udx/frn/f1.gml");
        write_file(
            &f1,
            &city_model(
                VALID_SRS_NAME_6697,
                &format!(
                    r#"    <core:cityObjectMember>
        <frn:CityFurniture gml:id="{frn1}">{frn_dm}
        </frn:CityFurniture>
    </core:cityObjectMember>
    <core:cityObjectMember>
        <veg:PlantCover gml:id="{veg2}">{veg_dm}
        </veg:PlantCover>
    </core:cityObjectMember>"#,
                    frn_dm = dm(&[
                        "<gml:Point/>",
                        "<gml:MultiPoint/>",
                        "<gml:MultiCurve/>",
                        "<gml:MultiSurface/>",
                        "<gml:Solid/>",
                        "<gml:CompositeSurface/>",
                    ]),
                    veg_dm = dm(&["<gml:LineString/>"]),
                ),
            ),
        )?;

        let outputs = run(vec![input("f1.gml", "frn", 0, &f1, Some(&codelists))], None)?;

        assert_eq!(
            fields(
                &outputs,
                "InvalidLodXGeometry",
                &["featureType", "gmlId", "parentTag", "invalidGeometry"]
            ),
            [
                ("CityFurniture", "", "urc:DmGeometricAttribute", "gml:Solid"),
                (
                    "CityFurniture",
                    "",
                    "urc:DmGeometricAttribute",
                    "gml:CompositeSurface"
                ),
                (
                    "PlantCover",
                    "",
                    "urc:DmGeometricAttribute",
                    "gml:LineString"
                ),
            ]
            .map(|row| <[&str; 4]>::from(row).map(|v| json!(v)).to_vec())
            .to_vec()
        );
        assert_eq!(
            fields(&outputs, "Summary", &["invalidLodXGeometry"]),
            [[json!(3)]]
        );
        Ok(())
    }

    #[test]
    // Without an objectlist, L03 counts the members of types outside the
    // package by local name.
    fn test_feature_types_outside_package() -> Result<(), BoxedError> {
        let temp_dir = TempDir::with_prefix("domain_of_definition_validator_test_")?;
        let root = temp_dir.path();
        let codelists = root.join("codelists");
        fs::create_dir_all(&codelists)?;
        let members = [
            "bldg:Building",
            "veg:PlantCover",
            "veg:PlantCover",
            "frn:CityFurniture",
            "con:OtherConstruction",
            "foo:Bar",
            "grp:CityObjectGroup",
        ]
        .iter()
        .enumerate()
        .map(|(i, tag)| {
            format!(
                r#"    <core:cityObjectMember>
        <{tag} gml:id="{}"/>
    </core:cityObjectMember>"#,
                gml_id("bldg", i as u32)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
        let f1 = root.join("udx/bldg/f1.gml");
        write_file(&f1, &city_model(VALID_SRS_NAME_6697, &members))?;

        let outputs = run(
            vec![input("f1.gml", "bldg", 0, &f1, Some(&codelists))],
            None,
        )?;

        assert_eq!(
            fields(
                &outputs,
                "Summary",
                &["invalidFeatureTypesNum", "invalidFeatureTypesDetail"]
            ),
            [[
                json!(5),
                json!({
                    "Bar": 1,
                    "CityFurniture": 1,
                    "OtherConstruction": 1,
                    "PlantCover": 2,
                })
            ]]
        );
        assert!(with_flag(&outputs, "ApplicationSchemaViolation").is_empty());
        Ok(())
    }

    #[test]
    // A file that cannot be parsed, and one whose member has no gml:id, still
    // have a summary. The members after the one without gml:id are not
    // checked, nor the xlinks of a city object group.
    fn test_summary_of_unprocessable_files() -> Result<(), BoxedError> {
        let temp_dir = TempDir::with_prefix("domain_of_definition_validator_test_")?;
        let root = temp_dir.path();
        let codelists = root.join("codelists");
        fs::create_dir_all(&codelists)?;
        let f1 = root.join("udx/bldg/f1.gml");
        write_file(
            &f1,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0" xmlns:gml="http://www.opengis.net/gml/3.2">
    <gml:boundedBy>
</core:CityModel>"#,
        )?;
        let grp = gml_id("grp", 1);
        let missing = gml_id("bldg", 99);
        let f2 = root.join("udx/bldg/f2.gml");
        write_file(
            &f2,
            &city_model(
                VALID_SRS_NAME_6697,
                &format!(
                    r#"{group}
    <core:cityObjectMember>
        <bldg:Building gml:id="bldg_not-a-uuid"/>
    </core:cityObjectMember>
    <core:cityObjectMember>
        <veg:PlantCover/>
    </core:cityObjectMember>
    <core:cityObjectMember>
        <bldg:Building gml:id="bldg_not-a-uuid-either"/>
    </core:cityObjectMember>"#,
                    group = city_object_group(&grp, &[format!("#{missing}")]),
                ),
            ),
        )?;

        let outputs = run(
            vec![
                input("f1.gml", "bldg", 0, &f1, Some(&codelists)),
                input("f2.gml", "bldg", 1, &f2, Some(&codelists)),
            ],
            None,
        )?;

        let expected = vec![
            feature_output(json!({
                "index": 0,
                "flag": "Summary",
                "srsName": "",
                "invalidFeatureTypesNum": 0,
                "invalidFeatureTypesDetail": {},
                "correctCodeValues": 0,
                "incorrectCodeValue": 0,
                "incorrectCodeSpace": 0,
                "isCorrectSrsName": true,
                "correctExtents": 0,
                "incorrectExtents": 0,
                "lowerLatitude": null, "lowerLongitude": null, "lowerElevation": null,
                "upperLatitude": null, "upperLongitude": null, "upperElevation": null,
                "gmlIdNotWellformed": 0,
                "xlinkHasNoReference": 0,
                "xlinkInvalidObjectType": 0,
                "invalidLodXGeometry": 0,
            })),
            feature_output(json!({
                "index": 1,
                "udxDirs": "bldg",
                "name": "f2.gml",
                "path": file_uri(&f2),
                "gmlId": "bldg_not-a-uuid",
                "featureType": "Building",
                "flag": "GMLID_NotWellFormed",
                "tag": "bldg:Building",
            })),
            feature_output(json!({
                "index": 1,
                "flag": "Summary",
                "srsName": VALID_SRS_NAME_6697,
                "invalidFeatureTypesNum": 1,
                "invalidFeatureTypesDetail": {"PlantCover": 1},
                "correctCodeValues": 0,
                "incorrectCodeValue": 0,
                "incorrectCodeSpace": 0,
                "isCorrectSrsName": true,
                "correctExtents": 0,
                "incorrectExtents": 0,
                "lowerLatitude": 36.6, "lowerLongitude": 137.0, "lowerElevation": 0.0,
                "upperLatitude": 36.7, "upperLongitude": 137.1, "upperElevation": 10.0,
                "gmlIdNotWellformed": 1,
                "xlinkHasNoReference": 0,
                "xlinkInvalidObjectType": 0,
                "invalidLodXGeometry": 0,
            })),
            stats_output("f1.gml", 0),
            stats_output("f2.gml", 0),
        ];
        assert_outputs(outputs, expected);
        Ok(())
    }
}
