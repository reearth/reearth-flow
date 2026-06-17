use core::fmt;
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Formatter},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use bytes::Bytes;
use fastxml::schema::fetcher::{DefaultFetcher, FetchResult, SchemaFetcher};
use fastxml::schema::types::CompiledSchema;
use fastxml::schema::xsd::{compile_schemas, register_builtin_types, SchemaResolver};
use once_cell::sync::Lazy;
use reearth_flow_common::{uri::Uri, xml};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{AttributeValue, Feature};
use serde_json::Value;

use super::errors::{Result, XmlProcessorError};
use super::types::{ValidationResult, ValidationType, XmlInputType, XmlValidatorParam};

static SUCCESS_PORT: Lazy<Port> = Lazy::new(|| Port::new("success"));
static FAILED_PORT: Lazy<Port> = Lazy::new(|| Port::new("failed"));

/// Persistent on-disk cache directory for fetched XSD schemas.
///
/// Remote GML/CityGML base schemas (e.g. `http://schemas.opengis.net/...`) are
/// referenced by absolute URL and are not shipped in the local schema bundle, so
/// without a persistent cache every validation run re-downloads them. Pointing the
/// `FileCachingFetcher` at a stable directory makes those downloads happen once and
/// be reused across documents, workflow runs, and processes.
///
/// Override the location with `FLOW_RUNTIME_XML_SCHEMA_CACHE_DIR`.
static SCHEMA_CACHE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let dir = std::env::var("FLOW_RUNTIME_XML_SCHEMA_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("reearth-flow-xml-schema-cache"));
    if let Err(e) = std::fs::create_dir_all(&dir) {
        tracing::warn!(dir = ?dir, error = ?e, "Failed to create XML schema cache directory");
    }
    dir
});

/// Per-URL locks used to coalesce concurrent fetches of the same schema URL
/// within a process ("single-flight").
///
/// On a cold runner the on-disk cache is empty, so every worker thread that
/// starts a workflow at once races to download the same handful of remote
/// XSDs (the CityGML/GML base schemas). Sixteen threads hitting
/// `schemas.opengis.net` simultaneously trips its rate limiter and turns a
/// few-second download into minutes of retries. Electing a single thread per
/// URL to do the fetch — while the rest wait and then read the file it wrote —
/// removes the herd without touching the warm-cache fast path.
///
/// The map only ever grows by the number of distinct schema URLs seen in a
/// process (a few dozen), so it is not a meaningful leak.
static FETCH_LOCKS: Lazy<parking_lot::Mutex<HashMap<String, Arc<parking_lot::Mutex<()>>>>> =
    Lazy::new(|| parking_lot::Mutex::new(HashMap::new()));

fn fetch_lock_for(url: &str) -> Arc<parking_lot::Mutex<()>> {
    let mut locks = FETCH_LOCKS.lock();
    Arc::clone(
        locks
            .entry(url.to_string())
            .or_insert_with(|| Arc::new(parking_lot::Mutex::new(()))),
    )
}

/// Read a previously cached schema from disk, if present.
///
/// Only a regular file is served: the cache directory lives under a shared
/// temp dir, so a symlink planted at the cache path must not be followed to
/// read arbitrary content elsewhere on the filesystem.
fn read_cached_schema(path: &Path, url: &str) -> Option<FetchResult> {
    let meta = std::fs::symlink_metadata(path).ok()?;
    if !meta.file_type().is_file() {
        return None;
    }
    let content = std::fs::read(path).ok()?;
    Some(FetchResult {
        content,
        final_url: url.to_string(),
        redirected: false,
    })
}

/// Atomically write a fetched schema to the disk cache.
///
/// Writing via a sibling temp file + rename means a pre-existing symlink at the
/// destination is not followed, and concurrent readers see either the old or
/// the new content, never a partial write.
fn write_cached_schema(path: &Path, content: &[u8]) {
    let Some(parent) = path.parent() else {
        return;
    };
    let tmp = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("cache"),
        std::process::id()
    ));
    if std::fs::write(&tmp, content).is_ok() {
        let _ = std::fs::rename(&tmp, path);
    }
}

/// Persistent, cross-process disk cache for fetched XSD schemas.
///
/// `fastxml`'s own `FileCachingFetcher` only consults its in-memory index, which
/// starts empty on every process, so its on-disk files are never re-read across
/// runs. This wrapper keys directly off a deterministic filename: a fresh process
/// finds the existing file on disk and skips the network entirely. Concurrent
/// misses for the same URL are coalesced into a single network fetch.
struct DiskCachingFetcher {
    inner: DefaultFetcher,
    dir: PathBuf,
}

impl DiskCachingFetcher {
    fn cache_path(&self, url: &str) -> PathBuf {
        use std::hash::{Hash, Hasher};
        // DefaultHasher uses fixed keys (not randomized), so the filename is
        // stable across processes and machines.
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        url.hash(&mut hasher);
        self.dir.join(format!("{:016x}.xsd", hasher.finish()))
    }

    /// Serve `url` from the disk cache, or fetch it via `fetch_uncached` and
    /// populate the cache. Concurrent callers for the same URL are serialized
    /// so only one runs `fetch_uncached`; the rest read the file it wrote.
    fn fetch_with_cache(
        &self,
        url: &str,
        fetch_uncached: impl FnOnce() -> fastxml::error::Result<FetchResult>,
    ) -> fastxml::error::Result<FetchResult> {
        let path = self.cache_path(url);

        // Fast path: serve directly from disk without taking any lock.
        if let Some(result) = read_cached_schema(&path, url) {
            return Ok(result);
        }

        // Cache miss: coalesce concurrent fetches of this URL (see FETCH_LOCKS).
        let url_lock = fetch_lock_for(url);
        let _guard = url_lock.lock();

        // Re-check: the elected thread may have populated the cache while we
        // waited for the lock.
        if let Some(result) = read_cached_schema(&path, url) {
            return Ok(result);
        }

        let result = fetch_uncached()?;

        // Only cache direct (non-redirected) responses so a cached entry's
        // `final_url` stays correct for relative-import resolution.
        if !result.redirected && result.final_url == url {
            write_cached_schema(&path, &result.content);
        }

        Ok(result)
    }
}

impl SchemaFetcher for DiskCachingFetcher {
    fn fetch(&self, url: &str) -> fastxml::error::Result<FetchResult> {
        self.fetch_with_cache(url, || self.inner.fetch(url))
    }
}

#[derive(Debug, Clone, Default)]
pub struct XmlValidatorFactory;

impl ProcessorFactory for XmlValidatorFactory {
    fn name(&self) -> &str {
        "XMLValidator"
    }

    fn description(&self) -> &str {
        "Validates XML documents against XSD schemas with success/failure routing"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(XmlValidatorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Transform"]
    }

    fn tags(&self) -> &[&'static str] {
        &["xml", "validate"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![SUCCESS_PORT.clone(), FAILED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: XmlValidatorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                XmlProcessorError::ValidatorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                XmlProcessorError::ValidatorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(XmlProcessorError::ValidatorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let process = XmlValidator {
            params,
            schema_cache: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        };
        Ok(Box::new(process))
    }
}

#[derive(Clone)]
pub struct XmlValidator {
    params: XmlValidatorParam,
    /// Cache of compiled schemas keyed by sorted schema locations.
    /// Shared across clones (threads) via Arc so that the ~27s compilation
    /// cost is paid only once per unique set of schema locations.
    schema_cache: Arc<parking_lot::RwLock<HashMap<String, Arc<CompiledSchema>>>>,
}

/// Build a deterministic cache key from schema locations.
fn schema_cache_key(locations: &[(String, String)]) -> String {
    let mut parts: Vec<String> = locations
        .iter()
        .map(|(ns, loc)| format!("{ns}={loc}"))
        .collect();
    parts.sort();
    parts.join("|")
}

impl Debug for XmlValidator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("XmlValidator")
            .field("params", &self.params)
            .finish()
    }
}

impl Processor for XmlValidator {
    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        match self.params.validation_type {
            ValidationType::Syntax => self.validate_syntax_only(ctx, fw)?,
            ValidationType::SyntaxAndNamespace => self.validate_syntax_and_namespace(ctx, fw)?,
            ValidationType::SyntaxAndSchema => self.validate_syntax_and_schema(ctx, fw)?,
        }
        Ok(())
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "XMLValidator"
    }
}

impl XmlValidator {
    fn validate_syntax_only(
        &self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<()> {
        let feature = &ctx.feature;
        let xml_content = self.get_xml_content(&ctx, feature)?;

        let Ok(document) = xml::parse(xml_content) else {
            Self::send_syntax_error(&ctx, fw, feature);
            return Ok(());
        };

        let Ok(_) = xml::get_root_node(&document) else {
            Self::send_syntax_error(&ctx, fw, feature);
            return Ok(());
        };

        fw.send(ctx.new_with_feature_and_port(feature.clone(), SUCCESS_PORT.clone()));
        Ok(())
    }

    fn validate_syntax_and_namespace(
        &self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<()> {
        let feature = &ctx.feature;
        let xml_bytes = self.get_xml_content_bytes(&ctx, feature)?;

        match Self::check_namespace_streaming(&xml_bytes) {
            Ok(result) => {
                if result.is_empty() {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), SUCCESS_PORT.clone()));
                } else {
                    let mut feature = feature.clone();
                    feature.insert(
                        "xmlError",
                        AttributeValue::Array(
                            result
                                .into_iter()
                                .map(|r| AttributeValue::Map(r.into()))
                                .collect::<Vec<_>>(),
                        ),
                    );
                    fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));
                }
            }
            Err(_) => {
                Self::send_syntax_error(&ctx, fw, feature);
            }
        }

        Ok(())
    }

    fn validate_syntax_and_schema(
        &self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<()> {
        let feature = &ctx.feature;
        // Get XML as bytes for streaming validation (more memory efficient)
        let xml_bytes = self.get_xml_content_bytes(&ctx, feature)?;

        match self.check_schema_streaming(feature, &xml_bytes) {
            Ok(result) => {
                if result.is_empty() {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), SUCCESS_PORT.clone()));
                } else {
                    let mut feature = feature.clone();
                    feature.insert(
                        "xmlError",
                        AttributeValue::Array(
                            result
                                .into_iter()
                                .map(|r| AttributeValue::Map(r.into()))
                                .collect::<Vec<_>>(),
                        ),
                    );
                    fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));
                }
            }
            Err(e) => {
                let mut feature = feature.clone();
                feature.insert(
                    "xmlError",
                    AttributeValue::Array(vec![AttributeValue::Map(
                        ValidationResult::new(
                            "SchemaError",
                            &format!("Schema validation failed: {e}"),
                        )
                        .into(),
                    )]),
                );
                fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));
            }
        }

        Ok(())
    }

    fn get_xml_content(&self, ctx: &ExecutorContext, feature: &Feature) -> Result<String> {
        match self.params.input_type {
            XmlInputType::File => {
                let uri = feature
                    .attributes
                    .get(&self.params.attribute)
                    .ok_or(XmlProcessorError::Validator("Required Uri".to_string()))?;
                let uri = match uri {
                    AttributeValue::String(s) => Uri::from_str(s)
                        .map_err(|_| XmlProcessorError::Validator("Invalid URI".to_string()))?,
                    _ => {
                        return Err(XmlProcessorError::Validator(
                            "Invalid Attribute".to_string(),
                        ))
                    }
                };
                let storage = ctx
                    .storage_resolver
                    .resolve(&uri)
                    .map_err(|e| XmlProcessorError::Validator(format!("{e:?}")))?;
                let content = storage
                    .get_sync(uri.path().as_path())
                    .map_err(|e| XmlProcessorError::Validator(format!("{e:?}")))?;
                String::from_utf8(content.to_vec())
                    .map_err(|_| XmlProcessorError::Validator("Invalid UTF-8".to_string()))
            }
            XmlInputType::Text => {
                let content = feature
                    .attributes
                    .get(&self.params.attribute)
                    .ok_or(XmlProcessorError::Validator("No Attribute".to_string()))?;
                let content = match content {
                    AttributeValue::String(s) => s,
                    _ => {
                        return Err(XmlProcessorError::Validator(
                            "Invalid Attribute".to_string(),
                        ))
                    }
                };
                Ok(content.to_string())
            }
        }
    }

    /// Get XML content as bytes for streaming validation (more memory efficient)
    fn get_xml_content_bytes(&self, ctx: &ExecutorContext, feature: &Feature) -> Result<Bytes> {
        let result = match self.params.input_type {
            XmlInputType::File => {
                let uri = feature
                    .attributes
                    .get(&self.params.attribute)
                    .ok_or(XmlProcessorError::Validator("Required Uri".to_string()))?;
                let uri = match uri {
                    AttributeValue::String(s) => Uri::from_str(s)
                        .map_err(|_| XmlProcessorError::Validator("Invalid URI".to_string()))?,
                    _ => {
                        return Err(XmlProcessorError::Validator(
                            "Invalid Attribute".to_string(),
                        ))
                    }
                };
                let storage = ctx
                    .storage_resolver
                    .resolve(&uri)
                    .map_err(|e| XmlProcessorError::Validator(format!("{e:?}")))?;
                let content = storage
                    .get_sync(uri.path().as_path())
                    .map_err(|e| XmlProcessorError::Validator(format!("{e:?}")))?;
                Ok(content)
            }
            XmlInputType::Text => {
                let content = feature
                    .attributes
                    .get(&self.params.attribute)
                    .ok_or(XmlProcessorError::Validator("No Attribute".to_string()))?;
                let content = match content {
                    AttributeValue::String(s) => s,
                    _ => {
                        return Err(XmlProcessorError::Validator(
                            "Invalid Attribute".to_string(),
                        ))
                    }
                };
                Ok(Bytes::from(content.as_bytes().to_vec()))
            }
        };
        result
    }

    fn send_syntax_error(ctx: &ExecutorContext, fw: &ProcessorChannelForwarder, feature: &Feature) {
        let mut feature = feature.clone();
        feature.insert(
            "xmlError",
            AttributeValue::Array(vec![AttributeValue::Map(
                ValidationResult::new("SyntaxError", "Invalid document structure").into(),
            )]),
        );
        fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));
    }

    fn check_namespace_streaming(xml_bytes: &[u8]) -> Result<Vec<ValidationResult>> {
        use quick_xml::events::Event;
        use quick_xml::Reader;

        let mut reader = Reader::from_reader(xml_bytes);
        reader.config_mut().trim_text(false);

        let mut buf = Vec::new();
        let mut declared_prefixes: HashSet<String> = HashSet::new();
        let mut has_default_ns = false;
        let mut errors: Vec<ValidationResult> = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    // Collect namespace declarations on this element before checking
                    // its own prefix. A prefix may be declared and used on the same
                    // element (e.g. `<xal:Address xmlns:xal="...">`), and a
                    // declaration on any ancestor stays in scope for its descendants,
                    // so accumulate prefixes across the whole document rather than
                    // honoring only the root element's declarations.
                    for attr in e.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                        if let Some(prefix) = key.strip_prefix("xmlns:") {
                            declared_prefixes.insert(prefix.to_string());
                        } else if key == "xmlns" {
                            has_default_ns = true;
                        }
                    }
                    let name_bytes = e.name();
                    let name = std::str::from_utf8(name_bytes.as_ref()).unwrap_or("");
                    if let Some((prefix, _)) = name.split_once(':') {
                        if !declared_prefixes.contains(prefix) {
                            errors.push(ValidationResult::new(
                                "NamespaceError",
                                &format!("No namespace declaration for {prefix}"),
                            ));
                        }
                    } else if !has_default_ns {
                        errors.push(ValidationResult::new(
                            "NamespaceError",
                            "No namespace declaration",
                        ));
                    }
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => {
                    return Err(XmlProcessorError::Validator(format!(
                        "Failed to parse XML: {e}"
                    )));
                }
            }
            buf.clear();
        }

        Ok(errors)
    }

    /// Streaming schema validation.
    fn check_schema_streaming(
        &self,
        feature: &Feature,
        xml_bytes: &[u8],
    ) -> Result<Vec<ValidationResult>> {
        use fastxml::schema::Validator;

        // Determine base directory for relative schema resolution
        let base_dir = self.get_xml_base_url(feature).and_then(|uri| {
            uri.path()
                .to_str()
                .map(PathBuf::from)
                .filter(|p| p.exists())
        });
        // --- Step 1: Extract schema locations (streaming, no DOM) ---
        let schema_locations =
            fastxml::parser::parse_schema_locations_from_reader(std::io::Cursor::new(xml_bytes))
                .map_err(|e| {
                    XmlProcessorError::Validator(format!("Failed to extract schema locations: {e}"))
                })?;

        if schema_locations.is_empty() {
            return Ok(Vec::new());
        }

        // --- Step 2: Fetch + resolve + compile schemas (with cache) ---
        let cache_key = schema_cache_key(&schema_locations);

        // Check cache first
        let cached = {
            let cache = self.schema_cache.read();
            cache.get(&cache_key).cloned()
        };

        let compiled = if let Some(compiled) = cached {
            compiled
        } else {
            // Cache miss - compile from scratch
            let inner = match &base_dir {
                Some(dir) => DefaultFetcher::with_base_dir(dir),
                None => DefaultFetcher::new(),
            };
            // Persistent on-disk cache: fetched remote schemas are reused across
            // documents/runs/processes instead of being re-downloaded each time.
            let fetcher = DiskCachingFetcher {
                inner,
                dir: SCHEMA_CACHE_DIR.clone(),
            };

            let mut resolver = SchemaResolver::new(&fetcher);
            for (_namespace, location) in &schema_locations {
                // Per W3C spec, xsi:schemaLocation URLs are hints.
                // Remote schemas that are unreachable (404, DNS failure, etc.)
                // are skipped. This may cause false positives if the skipped
                // schema defines types used elsewhere.
                let is_remote = location.starts_with("http://") || location.starts_with("https://");

                let fetch_result = match fetcher.fetch(location) {
                    Ok(r) => r,
                    Err(e) if is_remote => {
                        tracing::warn!(url = %location, error = ?e, "Skipping unreachable remote schema");
                        continue;
                    }
                    Err(e) => {
                        return Err(XmlProcessorError::Validator(format!(
                            "Failed to fetch schema {location}: {e:?}"
                        )));
                    }
                };

                let base_uri = &fetch_result.final_url;
                match resolver.resolve_entry(&fetch_result.content, base_uri) {
                    Ok(()) => {}
                    Err(e) if is_remote => {
                        tracing::warn!(url = %location, error = ?e, "Skipping unresolvable remote schema");
                        continue;
                    }
                    Err(e) => {
                        return Err(XmlProcessorError::Validator(format!(
                            "Failed to resolve schema imports for {location}: {e:?}"
                        )));
                    }
                }
            }
            let all_schemas = resolver.take_all_schemas();

            let mut compiled = compile_schemas(all_schemas).map_err(|e| {
                XmlProcessorError::Validator(format!("Failed to compile schemas: {e:?}"))
            })?;
            register_builtin_types(&mut compiled);
            let compiled = Arc::new(compiled);

            // Store in cache
            {
                let mut cache = self.schema_cache.write();
                cache.insert(cache_key, Arc::clone(&compiled));
            }

            compiled
        };

        // --- Step 3: Streaming validation ---
        let reader = std::io::BufReader::new(std::io::Cursor::new(xml_bytes));
        let errors = Validator::from_reader(reader)
            .schema(Arc::clone(&compiled))
            .run()
            .map_err(|e| {
                XmlProcessorError::Validator(format!("Streaming validation failed: {e:?}"))
            })?
            .into_entries();

        // Convert errors to ValidationResult and deduplicate
        let validation_results: HashSet<_> = errors
            .into_iter()
            .map(|err| {
                ValidationResult::new_with_line_and_col(
                    "SchemaError",
                    &err.message,
                    err.line().map(|l| l as i32),
                    err.column().map(|c| c as i32),
                )
            })
            .collect();

        Ok(validation_results.into_iter().collect())
    }

    fn get_xml_base_url(&self, feature: &Feature) -> Option<Uri> {
        match self.params.input_type {
            XmlInputType::File => feature
                .attributes
                .get(&self.params.attribute)
                .and_then(|v| {
                    if let AttributeValue::String(s) = v {
                        match Uri::from_str(s) {
                            Ok(uri) => {
                                if uri.is_dir() {
                                    Some(uri)
                                } else {
                                    uri.parent()
                                }
                            }
                            Err(_) => None,
                        }
                    } else {
                        None
                    }
                }),
            XmlInputType::Text => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{path::Path, sync::Arc};

    use super::*;
    use crate::tests::utils;
    use indexmap::IndexMap;
    use reearth_flow_runtime::forwarder::{NoopChannelForwarder, ProcessorChannelForwarder};
    use reearth_flow_types::{Attribute, AttributeValue, Feature, Geometry};

    #[cfg(not(feature = "new-geometry"))]
    #[test]
    fn test_xml_validator_syntax_validation() {
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<root>
    <element>test</element>
</root>"#;

        let (port, _features) = run_validator_test(xml_content, ValidationType::Syntax);
        assert_eq!(port, *SUCCESS_PORT, "Should output to success port");
    }

    #[cfg(not(feature = "new-geometry"))]
    #[test]
    fn test_xml_validator_invalid_syntax() {
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<root>
    <element>test</element>
    <unclosed_tag>
</root>"#;

        let (port, features) = run_validator_test(xml_content, ValidationType::Syntax);
        assert_eq!(
            port, *FAILED_PORT,
            "Should output to failed port for invalid XML"
        );

        // Verify error information is present
        match features[0].attributes.get(&Attribute::new("xmlError")) {
            Some(AttributeValue::Array(errors)) => {
                assert!(
                    !errors.is_empty(),
                    "Should have validation error information"
                );
                match errors.first() {
                    Some(AttributeValue::Map(error_map)) => {
                        assert!(
                            error_map.contains_key("errorType"),
                            "Should have error type"
                        );
                        assert!(
                            error_map.contains_key("message"),
                            "Should have error message"
                        );
                    }
                    _ => panic!("Expected error map in first array element"),
                }
            }
            _ => panic!("Should have xmlError attribute with validation errors"),
        }
    }

    #[cfg(not(feature = "new-geometry"))]
    #[test]
    fn test_xml_validator_malformed_xml() {
        let xml_content = r#"This is not XML at all!
<random>unclosed tag
&invalid;entity;
<>"#;

        let (port, features) = run_validator_test(xml_content, ValidationType::Syntax);
        assert_eq!(
            port, *FAILED_PORT,
            "Should output to failed port for malformed XML"
        );

        // Verify error information is present
        match features[0].attributes.get(&Attribute::new("xmlError")) {
            Some(AttributeValue::Array(errors)) => {
                assert!(
                    !errors.is_empty(),
                    "Should have validation error information"
                );
                match errors.first() {
                    Some(AttributeValue::Map(error_map)) => {
                        match error_map.get("errorType") {
                            Some(AttributeValue::String(error_type)) => {
                                assert_eq!(error_type, "SyntaxError", "Should be syntax error");
                            }
                            _ => panic!("Expected errorType to be a string"),
                        }
                        match error_map.get("message") {
                            Some(AttributeValue::String(message)) => {
                                assert_eq!(
                                    message, "Invalid document structure",
                                    "Should have proper error message"
                                );
                            }
                            _ => panic!("Expected message to be a string"),
                        }
                    }
                    _ => panic!("Expected error map in first array element"),
                }
            }
            _ => panic!("Should have xmlError attribute with validation errors"),
        }
    }

    #[cfg(not(feature = "new-geometry"))]
    #[test]
    fn test_xml_validator_missing_local_schema() {
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<root xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
      xsi:schemaLocation="http://example.com/schema ./test-schema.xsd">
    <element>test</element>
</root>"#;

        let (port, features) = run_validator_test(xml_content, ValidationType::SyntaxAndSchema);

        // Since local schema file doesn't exist, this should fail
        assert_eq!(
            port, *FAILED_PORT,
            "Should output to failed port for missing schema"
        );

        // Verify error information is present
        match features[0].attributes.get(&Attribute::new("xmlError")) {
            Some(AttributeValue::Array(errors)) => {
                assert!(
                    !errors.is_empty(),
                    "Should have validation error information"
                );
            }
            _ => panic!("Should have xmlError attribute with validation errors"),
        }
    }

    #[test]
    fn test_xml_validator_in_async_context() {
        // Verify that lazy initialization prevents panic when creating reqwest::blocking::Client in async context
        use reearth_flow_eval_expr::engine::Engine;
        use reearth_flow_runtime::{
            event::EventHub,
            executor_operation::{ExecutorContext, NodeContext},
            forwarder::{NoopChannelForwarder, ProcessorChannelForwarder},
            kvs::create_kv_store,
            node::ProcessorFactory,
        };
        use reearth_flow_storage::resolve::StorageResolver;

        // Create a runtime to simulate the actual execution environment
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        // Test that XmlValidator can be created and executed within runtime.block_on
        runtime.block_on(async {
            let factory = XmlValidatorFactory {};

            // Create required dependencies for NodeContext
            let expr_engine = Arc::new(Engine::new());
            let storage_resolver = Arc::new(StorageResolver::new());
            let kv_store = Arc::new(create_kv_store());
            let event_hub = EventHub::new(1024);

            let ctx = NodeContext::new(
                expr_engine.clone(),
                storage_resolver.clone(),
                kv_store.clone(),
                event_hub.clone(),
                reearth_flow_common::uri::Uri::for_test("file:///"),
            );

            let mut with = HashMap::new();
            with.insert(
                "attribute".to_string(),
                serde_json::Value::String("xml_content".to_string()),
            );
            with.insert(
                "inputType".to_string(),
                serde_json::Value::String("text".to_string()),
            );
            with.insert(
                "validationType".to_string(),
                serde_json::Value::String("syntaxAndSchema".to_string()),
            );

            // This should not panic with our lazy initialization
            let result = factory.build(ctx, event_hub, "xmlValidator".to_string(), Some(with));

            assert!(
                result.is_ok(),
                "Should be able to create XmlValidator in async context"
            );

            let mut processor = result.unwrap();

            // Create a feature with XML content that includes HTTPS schema reference
            let mut feature = Feature::new_with_attributes(IndexMap::new());
            feature.insert(
                Attribute::new("xml_content"),
                AttributeValue::String(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
                    <note xmlns="http://example.com/note"
                          xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                          xsi:schemaLocation="http://example.com/note https://raw.githubusercontent.com/reearth/reearth-flow/main/engine/runtime/action-processor/fixtures/xml/simple_note_schema.xsd">
                      <to>Tove</to>
                      <from>Jani</from>
                      <heading>Reminder</heading>
                      <body>Don't forget me this weekend!</body>
                    </note>"#.to_string()
                )
            );

            // Execute the processor in spawn_blocking to simulate real runtime behavior
            let handle = tokio::task::spawn_blocking(move || {
                let exec_ctx = ExecutorContext::new(
                    feature,
                    DEFAULT_PORT.clone(),
                    expr_engine,
                    storage_resolver,
                    kv_store,
                    EventHub::new(1024),
                    reearth_flow_common::uri::Uri::for_test("file:///"),
                );
                let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

                // This should not panic - HTTP client creation happens here in blocking context
                let result = processor.process(exec_ctx, &fw);

                (result, fw)
            });

            let (process_result, fw) = handle.await.unwrap();

            assert!(process_result.is_ok(), "Processing should complete without panic");

            // Check that we received output
            match &fw {
                ProcessorChannelForwarder::Noop(noop_fw) => {
                    let send_ports = noop_fw.send_ports.lock().unwrap();
                    assert!(!send_ports.is_empty(), "Should have sent output");

                    // The output should be on either success or failed port
                    assert!(
                        send_ports[0] == *SUCCESS_PORT || send_ports[0] == *FAILED_PORT,
                        "Should output to success or failed port"
                    );
                }
                _ => panic!("Expected Noop forwarder for testing"),
            }
        });
    }

    #[cfg(not(feature = "new-geometry"))]
    #[test]
    fn test_xml_validator_schema_unreachable_url_should_not_error() {
        // xsi:schemaLocation URLs are hints per W3C spec.
        // When a remote schema URL is unreachable (404, etc.), the validator
        // should skip it and succeed instead of routing to the failed port.
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<root xmlns="http://example.com/test"
      xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
      xsi:schemaLocation="http://example.com/test http://example.invalid/nonexistent.xsd">
    <element>test</element>
</root>"#;

        let (port, _features) = run_validator_test(xml_content, ValidationType::SyntaxAndSchema);
        assert_eq!(
            port, *SUCCESS_PORT,
            "Should succeed when remote schema URL is unreachable"
        );
    }

    #[test]
    fn test_disk_caching_fetcher_coalesces_concurrent_fetches() {
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc, Barrier,
        };

        // Each thread fetches the same URL through a temp-dir-backed disk cache.
        // The "network" fetch is a counting closure, so we can assert that the
        // single-flight lock collapses the herd into exactly one real fetch
        // (the rest are served from the file that fetch wrote). This is the
        // cold-runner scenario that previously stampeded schemas.opengis.net.
        let tmp = tempfile::tempdir().expect("create temp cache dir");
        let fetcher = Arc::new(DiskCachingFetcher {
            inner: DefaultFetcher::new(),
            dir: tmp.path().to_path_buf(),
        });

        // Unique URL so neither the process-global FETCH_LOCKS map nor the
        // per-test temp dir can be warmed by another test.
        let url = "http://example.test/coalesce/unique-schema-aa17.xsd";
        let network_calls = Arc::new(AtomicUsize::new(0));

        const THREADS: usize = 16;
        let barrier = Arc::new(Barrier::new(THREADS));

        let handles: Vec<_> = (0..THREADS)
            .map(|_| {
                let fetcher = Arc::clone(&fetcher);
                let network_calls = Arc::clone(&network_calls);
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    // Release all threads together so they genuinely contend.
                    barrier.wait();
                    fetcher
                        .fetch_with_cache(url, || {
                            network_calls.fetch_add(1, Ordering::SeqCst);
                            // Widen the race window so the late threads are
                            // forced through the lock + disk re-check path.
                            std::thread::sleep(std::time::Duration::from_millis(50));
                            Ok(FetchResult {
                                content: b"<xsd:schema/>".to_vec(),
                                final_url: url.to_string(),
                                redirected: false,
                            })
                        })
                        .expect("fetch_with_cache should succeed")
                })
            })
            .collect();

        let results: Vec<FetchResult> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        assert_eq!(
            network_calls.load(Ordering::SeqCst),
            1,
            "concurrent fetches of the same URL must coalesce into one network call"
        );
        for result in results {
            assert_eq!(
                result.content,
                b"<xsd:schema/>".to_vec(),
                "every caller must receive the fetched content"
            );
        }
    }

    #[cfg(not(feature = "new-geometry"))]
    // Regression test for L02 false positive:
    // "element 'bldg:opening' is not declared in schema" is incorrectly reported
    // for valid PLATEAU CityGML files. The root cause is that check_schema_streaming
    // creates a new SchemaResolver per xsi:schemaLocation entry, producing duplicate
    // schemas that corrupt the type-children cache and cause WallSurfaceType to lose
    // the inherited 'opening' element from AbstractBoundarySurfaceType.
    #[test]
    fn test_xml_validator_bldg_opening_false_positive_l02() {
        init_tracing();
        // Minimal valid PLATEAU CityGML with bldg:opening inside WallSurface.
        // The header and namespace declarations are copied from real data that
        // triggered the false positive (53394509_bldg_6697_op.gml).
        // Multiple schemaLocation entries are required to trigger the bug.
        let gml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel
    xmlns:app="http://www.opengis.net/citygml/appearance/2.0"
    xmlns:bldg="http://www.opengis.net/citygml/building/2.0"
    xmlns:brid="http://www.opengis.net/citygml/bridge/2.0"
    xmlns:core="http://www.opengis.net/citygml/2.0"
    xmlns:dem="http://www.opengis.net/citygml/relief/2.0"
    xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0"
    xmlns:gen="http://www.opengis.net/citygml/generics/2.0"
    xmlns:gml="http://www.opengis.net/gml"
    xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0"
    xmlns:luse="http://www.opengis.net/citygml/landuse/2.0"
    xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0"
    xmlns:sch="http://www.ascc.net/xml/schematron"
    xmlns:smil20="http://www.w3.org/2001/SMIL20/"
    xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language"
    xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0"
    xmlns:tran="http://www.opengis.net/citygml/transportation/2.0"
    xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0"
    xmlns:uro="https://www.geospatial.jp/iur/uro/3.2"
    xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0"
    xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0"
    xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0"
    xmlns:xlink="http://www.w3.org/1999/xlink"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="
        https://www.geospatial.jp/iur/uro/3.2 ../../schemas/iur/uro/3.2/urbanObject.xsd
        http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd
        http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd
        http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd
        http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd
        http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd
    ">
    <gml:boundedBy>
        <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
            <gml:lowerCorner>35.6 139.7 0</gml:lowerCorner>
            <gml:upperCorner>35.7 139.8 100</gml:upperCorner>
        </gml:Envelope>
    </gml:boundedBy>
    <core:cityObjectMember>
        <bldg:Building gml:id="bldg_00000000-0000-0000-0000-000000000001">
            <core:creationDate>2025-03-01</core:creationDate>
            <bldg:measuredHeight uom="m">10.0</bldg:measuredHeight>
            <bldg:lod0RoofEdge>
                <gml:MultiSurface>
                    <gml:surfaceMember>
                        <gml:Polygon>
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>35.6 139.7 0 35.6 139.71 0 35.61 139.71 0 35.61 139.7 0 35.6 139.7 0</gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>
                </gml:MultiSurface>
            </bldg:lod0RoofEdge>
            <bldg:boundedBy>
                <bldg:WallSurface gml:id="surface-00000000-0000-0000-0000-000000000001">
                    <bldg:lod3MultiSurface>
                        <gml:MultiSurface>
                            <gml:surfaceMember>
                                <gml:Polygon>
                                    <gml:exterior>
                                        <gml:LinearRing>
                                            <gml:posList>35.6 139.7 0 35.6 139.7 10 35.6 139.71 10 35.6 139.71 0 35.6 139.7 0</gml:posList>
                                        </gml:LinearRing>
                                    </gml:exterior>
                                </gml:Polygon>
                            </gml:surfaceMember>
                        </gml:MultiSurface>
                    </bldg:lod3MultiSurface>
                    <bldg:opening>
                        <bldg:Window gml:id="wnd_00000000-0000-0000-0000-000000000001">
                            <bldg:lod3MultiSurface>
                                <gml:MultiSurface>
                                    <gml:surfaceMember>
                                        <gml:Polygon>
                                            <gml:exterior>
                                                <gml:LinearRing>
                                                    <gml:posList>35.6 139.7 2 35.6 139.7 5 35.6 139.705 5 35.6 139.705 2 35.6 139.7 2</gml:posList>
                                                </gml:LinearRing>
                                            </gml:exterior>
                                        </gml:Polygon>
                                    </gml:surfaceMember>
                                </gml:MultiSurface>
                            </bldg:lod3MultiSurface>
                        </bldg:Window>
                    </bldg:opening>
                </bldg:WallSurface>
            </bldg:boundedBy>
        </bldg:Building>
    </core:cityObjectMember>
</core:CityModel>"#;

        let layout = PlateauTestLayout {
            udx_subdir: "bldg",
            gml_filename: "53394509_bldg_6697_op.gml",
            citymodel_name: "13101_chiyoda-ku_pref_2025_citygml_1_op",
        };
        let (_tmp_dir, port, features) =
            run_file_validator_test(gml_content, &layout, ValidationType::SyntaxAndSchema);

        // Extract validator error messages so they appear in the test output on failure
        let error_messages: Vec<String> = features
            .first()
            .and_then(|f| f.attributes.get(&Attribute::new("xmlError")))
            .and_then(|v| {
                if let AttributeValue::Array(arr) = v {
                    Some(arr)
                } else {
                    None
                }
            })
            .map(|errors| {
                errors
                    .iter()
                    .filter_map(|e| {
                        if let AttributeValue::Map(m) = e {
                            m.get("message").and_then(|v| {
                                if let AttributeValue::String(s) = v {
                                    tracing::error!("xmlError: {s}");
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        assert_eq!(
            port, *SUCCESS_PORT,
            "Valid CityGML with bldg:opening in WallSurface should pass schema validation.\n\
             Validator errors: {error_messages:?}"
        );
    }

    //
    // Test utilities
    //

    fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
            )
            .with_test_writer()
            .try_init();
    }

    fn create_xml_validator(validation_type: ValidationType) -> XmlValidator {
        let params = XmlValidatorParam {
            attribute: Attribute::new("xml_content"),
            input_type: XmlInputType::Text,
            validation_type,
        };

        XmlValidator {
            params,
            schema_cache: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }

    #[cfg(not(feature = "new-geometry"))]
    fn create_feature_with_xml(xml_content: &str) -> Feature {
        let mut attributes = IndexMap::new();
        attributes.insert(
            Attribute::new("xml_content"),
            AttributeValue::String(xml_content.to_string()),
        );

        Feature::new_with_attributes_and_geometry(attributes, Geometry::new())
    }

    #[cfg(not(feature = "new-geometry"))]
    fn run_validator_test(
        xml_content: &str,
        validation_type: ValidationType,
    ) -> (Port, Vec<Feature>) {
        let feature = create_feature_with_xml(xml_content);
        let mut validator = create_xml_validator(validation_type);

        let ctx = utils::create_default_execute_context(&feature);
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

        let result = validator.process(ctx, &fw);
        assert!(result.is_ok(), "XML validation processing should succeed");

        match fw {
            ProcessorChannelForwarder::Noop(noop_fw) => {
                let send_ports = noop_fw.send_ports.lock().unwrap();
                let send_features = noop_fw.send_features.lock().unwrap();
                assert!(!send_ports.is_empty(), "Should have sent output");

                (send_ports[0].clone(), send_features.clone())
            }
            _ => panic!("Expected Noop forwarder for testing"),
        }
    }

    /// Parameters for setting up a PLATEAU citymodel temp directory.
    struct PlateauTestLayout {
        /// GML filename (e.g. "53394509_bldg_6697_op.gml").
        gml_filename: &'static str,
        /// Subdirectory under `udx/` where the GML file is placed (e.g. "bldg", "tran").
        udx_subdir: &'static str,
        /// Fixture directory name under `testing/data/fixtures/plateau-citymodel/`.
        citymodel_name: &'static str,
    }

    #[cfg(not(feature = "new-geometry"))]
    /// Set up a temp directory mimicking the PLATEAU citymodel structure for
    /// file-based schema validation tests:
    ///   <tmp>/
    ///     udx/<subdir>/<filename>   (GML content written here)
    ///     codelists -> <fixture>/codelists
    ///     schemas   -> <fixture>/schemas
    ///
    /// Returns `(TempDir, Port, Vec<Feature>)` just like `run_validator_test`.
    /// The caller must keep `TempDir` alive so the temp files are not deleted.
    fn run_file_validator_test(
        gml_content: &str,
        layout: &PlateauTestLayout,
        validation_type: ValidationType,
    ) -> (tempfile::TempDir, Port, Vec<Feature>) {
        let tmp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let tmp_path = tmp_dir.path();

        let data_dir = tmp_path.join("udx").join(layout.udx_subdir);
        std::fs::create_dir_all(&data_dir).expect("Failed to create udx subdirectory");

        let fixture_base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../testing/data/fixtures/plateau-citymodel")
            .join(layout.citymodel_name);
        let fixture_base = fixture_base.canonicalize().expect("Fixture base not found");

        link_fixture_dir(&fixture_base.join("codelists"), &tmp_path.join("codelists"));
        link_fixture_dir(&fixture_base.join("schemas"), &tmp_path.join("schemas"));

        let gml_path = data_dir.join(layout.gml_filename);
        std::fs::write(&gml_path, gml_content).expect("Failed to write GML file");

        let params = XmlValidatorParam {
            attribute: Attribute::new("xml_path"),
            input_type: XmlInputType::File,
            validation_type,
        };

        let mut validator = XmlValidator {
            params,
            schema_cache: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        };

        let file_uri = format!("file://{}", gml_path.display());
        let mut attributes = IndexMap::new();
        attributes.insert(Attribute::new("xml_path"), AttributeValue::String(file_uri));
        let feature = Feature::new_with_attributes_and_geometry(attributes, Geometry::new());

        let ctx = utils::create_default_execute_context(&feature);
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

        let result = validator.process(ctx, &fw);
        assert!(result.is_ok(), "XML validation processing should succeed");

        match fw {
            ProcessorChannelForwarder::Noop(noop_fw) => {
                let send_ports = noop_fw.send_ports.lock().unwrap();
                let send_features = noop_fw.send_features.lock().unwrap();
                assert!(!send_ports.is_empty(), "Should have sent output");

                (tmp_dir, send_ports[0].clone(), send_features.clone())
            }
            _ => panic!("Expected Noop forwarder for testing"),
        }
    }

    /// Link (or copy on non-unix) a fixture subdirectory into the temp directory.
    fn link_fixture_dir(fixture_src: &Path, dest: &Path) {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(fixture_src, dest).unwrap_or_else(|e| {
                panic!(
                    "Failed to symlink {} -> {}: {e}",
                    fixture_src.display(),
                    dest.display()
                )
            });
        }
        #[cfg(not(unix))]
        {
            fn copy_dir_recursive(src: &Path, dst: &Path) {
                std::fs::create_dir_all(dst).expect("Failed to create directory");
                for entry in std::fs::read_dir(src).expect("Failed to read directory") {
                    let entry = entry.expect("Failed to read entry");
                    let dest_path = dst.join(entry.file_name());
                    if entry.file_type().expect("Failed to get file type").is_dir() {
                        copy_dir_recursive(&entry.path(), &dest_path);
                    } else {
                        std::fs::copy(entry.path(), &dest_path).expect("Failed to copy file");
                    }
                }
            }
            copy_dir_recursive(fixture_src, dest);
        }
    }
}
