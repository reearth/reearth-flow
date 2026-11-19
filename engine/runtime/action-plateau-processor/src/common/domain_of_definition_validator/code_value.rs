//! L04: the code values and their codeSpace.

use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use fastxml::transform::Transformer;
use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_storage::storage::Storage;
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use serde_json::Number;

use super::report::MemberReporter;
use super::traversal::CodeSpaceEntry;
use crate::citygml::GML_DICTIONARY_NS;
use crate::common::errors::{PlateauProcessorError, Result};

/// A codelist, as { code: description }.
pub(super) type Codelist = HashMap<String, String>;

/// Each codelist file, by its URI.
pub(super) type Codelists = HashMap<String, Arc<Codelist>>;

/// The codelist files loaded when not in [`Codelists`], by their URI; `None`
/// for a file that cannot be loaded.
pub(super) type LoadedCodelists = HashMap<String, Option<Arc<Codelist>>>;

/// Where the codelist of a codeSpace in a city GML is looked for.
pub(super) struct CodelistResolver<'a> {
    pub(super) codelists: &'a Codelists,
    pub(super) storage_resolver: &'a Arc<StorageResolver>,
    /// The `dirCodelists` of the input feature.
    pub(super) dir_codelists: Option<Uri>,
    /// The directory of the city GML; `None` when it has none.
    pub(super) city_gml_dir: Option<&'a Uri>,
    pub(super) fallback_codelists_path: Option<&'a Uri>,
}

impl CodelistResolver<'_> {
    /// The codelist of `code_space`, or `None` when there is none. A codelist
    /// not in [`Codelists`] is loaded once into `loaded`.
    fn resolve(
        &self,
        base_dir: &Uri,
        code_space: &str,
        loaded: &mut LoadedCodelists,
    ) -> Result<Option<Arc<Codelist>>> {
        let mut load = |uri: &Uri| {
            loaded
                .entry(uri.to_string())
                .or_insert_with(|| {
                    try_load_codelist_file(Arc::clone(self.storage_resolver), uri)
                        .ok()
                        .map(Arc::new)
                })
                .clone()
        };

        // First, try to resolve using dirCodelists if codeSpace contains "codelists/"
        // This handles cases where GML files are extracted to a temp directory but codelists are elsewhere
        if let Some(codelists_relative) = extract_codelists_relative_path(code_space)
            && let Some(dir_codelists) = &self.dir_codelists
        {
            // Build the expected path by finding matching entry in codelists dictionary
            // We search for entries ending with the relative path since URI formats may vary
            let suffix = format!("/{}", codelists_relative);
            if let Some((_, code)) = self.codelists.iter().find(|(k, _)| k.ends_with(&suffix)) {
                return Ok(Some(Arc::clone(code)));
            }

            if let Ok(resolved_path) = dir_codelists.join(&codelists_relative) {
                // If not found by suffix, try direct join
                if let Some(code) = self.codelists.get(&resolved_path.to_string()) {
                    return Ok(Some(Arc::clone(code)));
                }

                // If still not found in pre-loaded dictionary, try to load the codelist file dynamically
                // This handles cases where the codelists dictionary was loaded from a different directory
                if let Some(code) = load(&resolved_path) {
                    return Ok(Some(code));
                }
            }
        }

        // Fallback: Try primary lookup (relative to GML file)
        let code_space_path = base_dir
            .join(Path::new(code_space))
            .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
        if let Some(code) = self.codelists.get(&code_space_path.to_string()) {
            return Ok(Some(Arc::clone(code)));
        }

        // If still not found and fallback path is provided, try the fallback path
        if let Some(fallback_path) = self.fallback_codelists_path
            // Extract the filename from codeSpace (e.g., "../../codelists/Building_class.xml" -> "Building_class.xml")
            && let Some(filename) = Path::new(code_space).file_name()
        {
            let fallback_code_space_path =
                fallback_path.join(Path::new(filename)).map_err(|e| {
                    PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}"))
                })?;
            // First try dictionary lookup
            if let Some(code) = self.codelists.get(&fallback_code_space_path.to_string()) {
                return Ok(Some(Arc::clone(code)));
            }

            // If not in dictionary, try dynamic loading
            if let Some(code) = load(&fallback_code_space_path) {
                return Ok(Some(code));
            }
        }
        Ok(None)
    }
}

/// The L04 check of a file.
pub(super) struct CodeValueCheck<'a> {
    correct_code_values: u32,
    code_value_errors: u32,
    code_space_errors: u32,
    /// The codelist of each codeSpace of the file, as resolved.
    resolved: HashMap<String, Option<Arc<Codelist>>>,
    /// Shared by all files.
    loaded: &'a mut LoadedCodelists,
}

impl<'a> CodeValueCheck<'a> {
    pub(super) fn new(loaded: &'a mut LoadedCodelists) -> Self {
        Self {
            correct_code_values: 0,
            code_value_errors: 0,
            code_space_errors: 0,
            resolved: HashMap::new(),
            loaded,
        }
    }

    pub(super) fn check_member(
        &mut self,
        resolver: &CodelistResolver,
        entries: &[CodeSpaceEntry],
        report: &MemberReporter,
    ) -> Result<()> {
        let base_dir =
            resolver
                .city_gml_dir
                .ok_or(PlateauProcessorError::DomainOfDefinitionValidator(
                    "illegal city gml path".to_string(),
                ))?;
        for entry in entries {
            let code = match self.resolved.get(&entry.code_space) {
                Some(code) => code.clone(),
                None => {
                    let code = resolver.resolve(base_dir, &entry.code_space, self.loaded)?;
                    self.resolved.insert(entry.code_space.clone(), code.clone());
                    code
                }
            };
            let flag = match code {
                Some(code) if code.contains_key(entry.content.as_str()) => {
                    self.correct_code_values += 1;
                    continue;
                }
                Some(_) => {
                    self.code_value_errors += 1;
                    "CodeValidation"
                }
                None => {
                    self.code_space_errors += 1;
                    "incorrectCodeSpace"
                }
            };
            report.send(
                flag,
                [
                    ("tag", entry.tag.clone()),
                    ("xpath", entry.xpath.clone()),
                    ("codeSpace", entry.code_space.clone()),
                    ("codeSpaceValue", entry.content.clone()),
                ],
            );
        }
        Ok(())
    }

    pub(super) fn write_summary(&self, summary: &mut Feature) {
        summary.insert(
            "correctCodeValues",
            AttributeValue::Number(Number::from(self.correct_code_values)),
        );
        summary.insert(
            "incorrectCodeValue",
            AttributeValue::Number(Number::from(self.code_value_errors)),
        );
        summary.insert(
            "incorrectCodeSpace",
            AttributeValue::Number(Number::from(self.code_space_errors)),
        );
    }
}

/// Extract the relative path after "codelists/" from a codeSpace path
/// e.g., "../../codelists/Building_class.xml" -> "Building_class.xml"
fn extract_codelists_relative_path(code_space: &str) -> Option<String> {
    const CODELISTS_DIR: &str = "codelists/";
    code_space
        .find(CODELISTS_DIR)
        .map(|idx| code_space[idx + CODELISTS_DIR.len()..].to_string())
}

/// Get the dirCodelists attribute from a feature if it exists
pub(super) fn get_dir_codelists(feature: &Feature) -> Option<Uri> {
    feature
        .attributes
        .get(&Attribute::new("dirCodelists"))
        .and_then(|v| Uri::from_str(&v.to_string()).ok())
}

/// Try to load a single codelist file dynamically from storage
/// Returns the codelist as a HashMap if successful, or an error if the file cannot be loaded
fn try_load_codelist_file(
    storage_resolver: Arc<StorageResolver>,
    codelist_path: &Uri,
) -> Result<Codelist> {
    let storage = storage_resolver.resolve(codelist_path).map_err(|e| {
        PlateauProcessorError::DomainOfDefinitionValidator(format!(
            "Failed to resolve codelist path: {e:?}"
        ))
    })?;

    if !storage
        .exists_sync(codelist_path.path().as_path())
        .unwrap_or(false)
    {
        return Err(PlateauProcessorError::DomainOfDefinitionValidator(format!(
            "Codelist file not found: {}",
            codelist_path
        )));
    }

    create_detail_codelist(storage, codelist_path)
}

pub(super) fn create_codelist(
    storage_resolver: Arc<StorageResolver>,
    first: &Feature,
    fallback_codelists_path: Option<&Uri>,
) -> Result<Codelists> {
    // Try to get dirCodelists from feature, fall back to provided path
    let dir = first
        .attributes
        .get(&Attribute::new("dirCodelists"))
        .and_then(|v| Uri::from_str(&v.to_string()).ok());

    let dir = match dir {
        Some(d) => {
            let storage = storage_resolver.resolve(&d).map_err(|e| {
                PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}"))
            })?;
            let exist = storage.exists_sync(d.path().as_path()).map_err(|e| {
                PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}"))
            })?;
            if exist {
                d
            } else if let Some(fallback) = fallback_codelists_path {
                fallback.clone()
            } else {
                return Err(PlateauProcessorError::DomainOfDefinitionValidator(
                    "dirCodelists not found and no fallback provided".to_string(),
                ));
            }
        }
        None => {
            if let Some(fallback) = fallback_codelists_path {
                fallback.clone()
            } else {
                return Err(PlateauProcessorError::DomainOfDefinitionValidator(
                    "dirCodelists key empty and no fallback provided".to_string(),
                ));
            }
        }
    };

    let storage = storage_resolver
        .resolve(&dir)
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    let exist = storage
        .exists_sync(dir.path().as_path())
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    if !exist {
        return Err(PlateauProcessorError::DomainOfDefinitionValidator(
            "codelists directory not found".to_string(),
        ));
    }
    let mut codelist = HashMap::new();
    let lists = storage
        .list_sync(Some(dir.path().as_path()), true)
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    for file in lists
        .iter()
        .filter(|f| f.is_file() && f.path().extension() == Some("xml".as_ref()))
    {
        let result = create_detail_codelist(Arc::clone(&storage), file)
            .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
        codelist.insert(file.to_string(), Arc::new(result));
    }
    Ok(codelist)
}

fn create_detail_codelist(storage: Arc<Storage>, xml_path: &Uri) -> Result<Codelist> {
    let xml_content = storage
        .get_sync(xml_path.path().as_path())
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    let xml_str = String::from_utf8(xml_content.to_vec())
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    let mut result = HashMap::new();
    let transformer = Transformer::from(xml_str.as_str())
        .with_root_namespaces()
        .map_err(|e| {
            PlateauProcessorError::DomainOfDefinitionValidator(format!(
                "Failed to parse root namespaces: {e:?}"
            ))
        })?;
    // Codelist dictionaries use the GML 3.1.1 SimpleDictionary profile namespace
    // regardless of the CityGML version, so use the fixed value rather than the
    // feature gml_ns.
    let definition_xpath =
        format!("//*[namespace-uri()='{GML_DICTIONARY_NS}'][local-name()='Definition']");
    transformer
        .on(definition_xpath.as_str(), |node| {
            let mut name_val = None;
            let mut desc_val = None;
            for child in node.children() {
                match child.qname().as_str() {
                    "gml:name" => name_val = child.get_content(),
                    "gml:description" => desc_val = child.get_content(),
                    _ => {}
                }
            }
            if let (Some(name), Some(desc)) = (name_val, desc_val) {
                result.insert(name, desc);
            }
        })
        .for_each()
        .map_err(|e| {
            PlateauProcessorError::DomainOfDefinitionValidator(format!(
                "Failed to parse codelist: {e:?}"
            ))
        })?;
    Ok(result)
}
