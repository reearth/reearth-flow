//! L04: the code values and their codeSpace.

use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use fastxml::transform::{TransformError, Transformer};
use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_storage::storage::Storage;
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use serde_json::Number;

/// An error of the code value check.
#[derive(thiserror::Error, Debug)]
pub(super) enum Error {
    #[error("the city GML path has no directory")]
    NoCityGmlDirectory,
    #[error("failed to join the codeSpace {code_space} to the directory of the city GML: {source}")]
    CodeSpacePath {
        code_space: String,
        #[source]
        source: reearth_flow_common::Error,
    },
    #[error("failed to join {file_name} to codelistsPath: {source}")]
    CodelistsPathJoin {
        file_name: String,
        #[source]
        source: reearth_flow_common::Error,
    },
    #[error("failed to resolve the storage of the codelist {path}: {source}")]
    CodelistStorage {
        path: String,
        #[source]
        source: reearth_flow_storage::Error,
    },
    #[error("the codelist file {0} does not exist")]
    CodelistNotFound(String),
    #[error("failed to read the codelist {path}: {source}")]
    CodelistRead {
        path: String,
        #[source]
        source: object_store::Error,
    },
    #[error("the codelist {path} is not UTF-8: {source}")]
    CodelistNotUtf8 {
        path: String,
        #[source]
        source: std::str::Utf8Error,
    },
    #[error("failed to parse the codelist {path}: {source}")]
    CodelistParse {
        path: String,
        #[source]
        source: TransformError,
    },
    #[error("failed to resolve the storage of dirCodelists {path}: {source}")]
    DirCodelistsStorage {
        path: String,
        #[source]
        source: reearth_flow_storage::Error,
    },
    #[error("failed to check whether dirCodelists {path} exists: {source}")]
    DirCodelistsAccess {
        path: String,
        #[source]
        source: object_store::Error,
    },
    #[error("dirCodelists does not exist and codelistsPath is not given")]
    DirCodelistsNotFound,
    #[error("the input feature has no dirCodelists and codelistsPath is not given")]
    DirCodelistsMissing,
    #[error("failed to resolve the storage of the codelists directory {path}: {source}")]
    CodelistsDirectoryStorage {
        path: String,
        #[source]
        source: reearth_flow_storage::Error,
    },
    #[error("failed to check whether the codelists directory {path} exists: {source}")]
    CodelistsDirectoryAccess {
        path: String,
        #[source]
        source: object_store::Error,
    },
    #[error("the codelists directory {0} does not exist")]
    CodelistsDirectoryNotFound(String),
    #[error("failed to list the codelists directory {path}: {source}")]
    CodelistsDirectoryList {
        path: String,
        #[source]
        source: object_store::Error,
    },
}

type Result<T, E = Error> = std::result::Result<T, E>;

use super::report::MemberReporter;
use super::traversal::CodeSpaceEntry;
use crate::citygml::GML_DICTIONARY_NS;

/// A codelist, as { code: description }.
pub(super) type Codelist = HashMap<String, String>;

/// Each codelist file, by its URI.
pub(super) type Codelists = HashMap<String, Arc<Codelist>>;

/// The codelist files loaded when not in [`Codelists`], by their URI. A file
/// that cannot be loaded is not kept, so that a later file tries it again.
pub(super) type LoadedCodelists = HashMap<String, Arc<Codelist>>;

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
    /// not in [`Codelists`] is loaded into `loaded`.
    fn resolve(
        &self,
        base_dir: &Uri,
        code_space: &str,
        loaded: &mut LoadedCodelists,
    ) -> Result<Option<Arc<Codelist>>> {
        let mut load = |uri: &Uri| {
            let key = uri.to_string();
            if let Some(code) = loaded.get(&key) {
                return Some(Arc::clone(code));
            }
            let code =
                Arc::new(try_load_codelist_file(Arc::clone(self.storage_resolver), uri).ok()?);
            loaded.insert(key, Arc::clone(&code));
            Some(code)
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
        let code_space_path =
            base_dir
                .join(Path::new(code_space))
                .map_err(|source| Error::CodeSpacePath {
                    code_space: code_space.to_string(),
                    source,
                })?;
        if let Some(code) = self.codelists.get(&code_space_path.to_string()) {
            return Ok(Some(Arc::clone(code)));
        }

        // If still not found and fallback path is provided, try the fallback path
        if let Some(fallback_path) = self.fallback_codelists_path
            // Extract the filename from codeSpace (e.g., "../../codelists/Building_class.xml" -> "Building_class.xml")
            && let Some(filename) = Path::new(code_space).file_name()
        {
            let fallback_code_space_path =
                fallback_path.join(Path::new(filename)).map_err(|source| {
                    Error::CodelistsPathJoin {
                        file_name: filename.to_string_lossy().to_string(),
                        source,
                    }
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
        let base_dir = resolver.city_gml_dir.ok_or(Error::NoCityGmlDirectory)?;
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
    let storage =
        storage_resolver
            .resolve(codelist_path)
            .map_err(|source| Error::CodelistStorage {
                path: codelist_path.to_string(),
                source,
            })?;

    if !storage
        .exists_sync(codelist_path.path().as_path())
        .unwrap_or(false)
    {
        return Err(Error::CodelistNotFound(codelist_path.to_string()));
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
            let storage =
                storage_resolver
                    .resolve(&d)
                    .map_err(|source| Error::DirCodelistsStorage {
                        path: d.to_string(),
                        source,
                    })?;
            let exist = storage.exists_sync(d.path().as_path()).map_err(|source| {
                Error::DirCodelistsAccess {
                    path: d.to_string(),
                    source,
                }
            })?;
            if exist {
                d
            } else if let Some(fallback) = fallback_codelists_path {
                fallback.clone()
            } else {
                return Err(Error::DirCodelistsNotFound);
            }
        }
        None => {
            if let Some(fallback) = fallback_codelists_path {
                fallback.clone()
            } else {
                return Err(Error::DirCodelistsMissing);
            }
        }
    };

    let path = || dir.to_string();
    let storage =
        storage_resolver
            .resolve(&dir)
            .map_err(|source| Error::CodelistsDirectoryStorage {
                path: path(),
                source,
            })?;
    let exist = storage
        .exists_sync(dir.path().as_path())
        .map_err(|source| Error::CodelistsDirectoryAccess {
            path: path(),
            source,
        })?;
    if !exist {
        return Err(Error::CodelistsDirectoryNotFound(path()));
    }
    let mut codelist = HashMap::new();
    let lists = storage
        .list_sync(Some(dir.path().as_path()), true)
        .map_err(|source| Error::CodelistsDirectoryList {
            path: path(),
            source,
        })?;
    for file in lists
        .iter()
        .filter(|f| f.is_file() && f.path().extension() == Some("xml".as_ref()))
    {
        let result = create_detail_codelist(Arc::clone(&storage), file)?;
        codelist.insert(file.to_string(), Arc::new(result));
    }
    Ok(codelist)
}

fn create_detail_codelist(storage: Arc<Storage>, xml_path: &Uri) -> Result<Codelist> {
    let path = || xml_path.to_string();
    let xml_content = storage
        .get_sync(xml_path.path().as_path())
        .map_err(|source| Error::CodelistRead {
            path: path(),
            source,
        })?;
    let xml_str = std::str::from_utf8(&xml_content).map_err(|source| Error::CodelistNotUtf8 {
        path: path(),
        source,
    })?;
    let mut result = HashMap::new();
    let transformer = Transformer::from(xml_str)
        .with_root_namespaces()
        .map_err(|source| Error::CodelistParse {
            path: path(),
            source,
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
        .map_err(|source| Error::CodelistParse {
            path: path(),
            source,
        })?;
    Ok(result)
}
