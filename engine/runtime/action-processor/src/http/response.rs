use base64::{engine::general_purpose, Engine as _};
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;

use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{Attribute, AttributeValue, Feature};

use super::client::HttpResponse;
use super::errors::{HttpProcessorError, Result};
use super::expression::CompiledResponseHandling;
use super::params::ResponseEncoding;

/// Attributes with fixed names, always written alongside the response body.
pub(crate) const STATUS_CODE_ATTRIBUTE: &str = "_http_status_code";
pub(crate) const HEADERS_ATTRIBUTE: &str = "_headers";
pub(crate) const ERROR_ATTRIBUTE: &str = "_http_error";
pub(crate) const FILE_PATH_ATTRIBUTE: &str = "_response_file_path";

pub(crate) struct ResponseProcessorConfig<'a> {
    pub handling: &'a Option<CompiledResponseHandling>,
    pub encoding: &'a Option<ResponseEncoding>,
    pub auto_detect: bool,
    pub variables: Arc<serde_json::Map<String, serde_json::Value>>,
    pub storage_resolver: &'a Arc<StorageResolver>,
    pub sandbox_root: &'a Uri,
    pub response_body_attr: &'a str,
}

pub(crate) fn process_response(
    response: HttpResponse,
    config: &ResponseProcessorConfig,
    feature: &Feature,
    attributes: &mut indexmap::IndexMap<Attribute, AttributeValue>,
) -> Result<()> {
    attributes.insert(
        Attribute::new(STATUS_CODE_ATTRIBUTE.to_string()),
        AttributeValue::Number(response.status_code.into()),
    );

    let headers_map: HashMap<String, AttributeValue> = response
        .headers
        .iter()
        .map(|(k, v)| (k.clone(), AttributeValue::String(v.clone())))
        .collect();
    attributes.insert(
        Attribute::new(HEADERS_ATTRIBUTE.to_string()),
        AttributeValue::Map(headers_map),
    );

    // An explicitly configured encoding always wins; detection from the
    // response's Content-Type only fills in when no encoding is set.
    let effective_encoding = config
        .encoding
        .clone()
        .or_else(|| {
            if config.auto_detect {
                detect_encoding_from_headers(&response.headers)
            } else {
                None
            }
        })
        .unwrap_or(ResponseEncoding::Text);

    match config
        .handling
        .as_ref()
        .unwrap_or(&CompiledResponseHandling::Attribute)
    {
        CompiledResponseHandling::Attribute => {
            let encoded_body = encode_response_body(&response.body, &effective_encoding);
            attributes.insert(
                Attribute::new(config.response_body_attr.to_string()),
                AttributeValue::String(encoded_body),
            );
        }
        CompiledResponseHandling::File { path_ast } => {
            let output_path = path_ast
                .eval_string(feature, config.variables.clone())
                .map_err(|e| {
                    HttpProcessorError::Response(format!("Failed to evaluate output path: {e:?}"))
                })?;

            let resolved = resolve_sandboxed_path(config.sandbox_root, &output_path)?;
            save_response_to_file(&response.body, &resolved, config.storage_resolver)?;

            attributes.insert(
                Attribute::new(FILE_PATH_ATTRIBUTE.to_string()),
                AttributeValue::String(resolved.to_string()),
            );
        }
    }

    Ok(())
}

/// Validate `path` as a strict-relative output path and resolve it against the
/// job's sandbox root. Mirrors the sink-output chokepoint rules: rejects
/// absolute URIs, absolute paths, home expansion, and any traversal segment.
fn resolve_sandboxed_path(sandbox_root: &Uri, path: &str) -> Result<Uri> {
    let reject = |reason: &str| {
        Err(HttpProcessorError::Response(format!(
            "Invalid response file path {path:?}: {reason}. Provide a relative \
             path under the job's output directory, like 'downloads/data.json'"
        )))
    };

    if path.is_empty() {
        return reject("path is empty");
    }
    if path != path.trim() {
        return reject("path has leading or trailing whitespace");
    }
    if path.contains("://") {
        return reject("absolute URIs are not allowed");
    }
    if path.starts_with('/') {
        return reject("absolute paths are not allowed");
    }
    if path.starts_with('~') {
        return reject("home expansion is not supported");
    }
    if path
        .split('/')
        .any(|segment| segment == ".." || segment == "." || segment.is_empty())
    {
        return reject("path traversal segments are not allowed");
    }

    sandbox_root.join(path).map_err(|e| {
        HttpProcessorError::Response(format!(
            "Failed to resolve response file path {path:?}: {e}"
        ))
    })
}

fn encode_response_body(body: &[u8], encoding: &ResponseEncoding) -> String {
    match encoding {
        ResponseEncoding::Text => String::from_utf8(body.to_vec())
            .unwrap_or_else(|_| String::from_utf8_lossy(body).into_owned()),
        ResponseEncoding::Base64 => general_purpose::STANDARD.encode(body),
    }
}

fn detect_encoding_from_headers(
    headers: &std::collections::HashMap<String, String>,
) -> Option<ResponseEncoding> {
    let content_type = headers
        .iter()
        .find(|(k, _)| k.to_lowercase() == "content-type")
        .map(|(_, v)| v.to_lowercase())?;

    if content_type.contains("text/")
        || content_type.contains("application/json")
        || content_type.contains("application/xml")
        || content_type.contains("application/javascript")
    {
        Some(ResponseEncoding::Text)
    } else if content_type.contains("image/")
        || content_type.contains("application/octet-stream")
        || content_type.contains("application/pdf")
        || content_type.contains("video/")
        || content_type.contains("audio/")
    {
        Some(ResponseEncoding::Base64)
    } else {
        None
    }
}

fn save_response_to_file(
    body: &[u8],
    uri: &Uri,
    storage_resolver: &Arc<StorageResolver>,
) -> Result<()> {
    let storage = storage_resolver.resolve(uri).map_err(|e| {
        HttpProcessorError::Response(format!("Failed to resolve storage path '{uri}': {e}"))
    })?;

    let path_string = uri.path().as_path().display().to_string();
    let storage_path = std::path::Path::new(&path_string);

    let bytes = Bytes::from(body.to_vec());

    storage.put_sync(storage_path, bytes).map_err(|e| {
        HttpProcessorError::Response(format!("Failed to save response to file '{uri}': {e}"))
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_types::Attributes;
    use std::str::FromStr;

    fn make_env() -> Arc<serde_json::Map<String, serde_json::Value>> {
        Arc::new(serde_json::Map::new())
    }

    fn empty_feature() -> Feature {
        Feature::from(Attributes::new())
    }

    fn sandbox_root() -> Uri {
        Uri::from_str("file:///tmp/job-artifacts").unwrap()
    }

    #[test]
    fn test_fixed_attributes_and_headers_map() {
        let response = HttpResponse {
            status_code: 200,
            headers: std::collections::HashMap::from([(
                "content-type".to_string(),
                "application/json".to_string(),
            )]),
            body: r#"{"ok":true}"#.as_bytes().to_vec(),
        };

        let storage_resolver = Arc::new(StorageResolver::new());
        let mut attributes = indexmap::IndexMap::new();
        let feature = empty_feature();
        let root = sandbox_root();

        let config = ResponseProcessorConfig {
            handling: &None,
            encoding: &None,
            auto_detect: true,
            variables: make_env(),
            storage_resolver: &storage_resolver,
            sandbox_root: &root,
            response_body_attr: "_response_body",
        };

        process_response(response, &config, &feature, &mut attributes).unwrap();

        assert!(matches!(
            attributes.get(&Attribute::new(STATUS_CODE_ATTRIBUTE.to_string())),
            Some(AttributeValue::Number(_))
        ));
        let Some(AttributeValue::Map(headers)) =
            attributes.get(&Attribute::new(HEADERS_ATTRIBUTE.to_string()))
        else {
            panic!("headers must be stored as a map");
        };
        assert!(matches!(
            headers.get("content-type"),
            Some(AttributeValue::String(v)) if v == "application/json"
        ));
        assert!(matches!(
            attributes.get(&Attribute::new("_response_body".to_string())),
            Some(AttributeValue::String(v)) if v == r#"{"ok":true}"#
        ));
    }

    #[test]
    fn test_explicit_encoding_wins_over_detection() {
        // Content-Type says text, but the user asked for base64: base64 wins.
        let response = HttpResponse {
            status_code: 200,
            headers: std::collections::HashMap::from([(
                "content-type".to_string(),
                "text/plain".to_string(),
            )]),
            body: b"hello".to_vec(),
        };

        let storage_resolver = Arc::new(StorageResolver::new());
        let mut attributes = indexmap::IndexMap::new();
        let feature = empty_feature();
        let root = sandbox_root();

        let config = ResponseProcessorConfig {
            handling: &None,
            encoding: &Some(ResponseEncoding::Base64),
            auto_detect: true,
            variables: make_env(),
            storage_resolver: &storage_resolver,
            sandbox_root: &root,
            response_body_attr: "_response_body",
        };

        process_response(response, &config, &feature, &mut attributes).unwrap();

        assert!(matches!(
            attributes.get(&Attribute::new("_response_body".to_string())),
            Some(AttributeValue::String(v)) if v == &general_purpose::STANDARD.encode(b"hello")
        ));
    }

    #[test]
    fn test_detect_binary_content_type_uses_base64() {
        let mut headers = std::collections::HashMap::new();
        headers.insert("Content-Type".to_string(), "image/png".to_string());

        let encoding = detect_encoding_from_headers(&headers);
        assert!(matches!(encoding, Some(ResponseEncoding::Base64)));
    }

    #[test]
    fn test_sandboxed_path_accepts_relative() {
        let root = sandbox_root();
        let resolved = resolve_sandboxed_path(&root, "downloads/data.json").unwrap();
        assert_eq!(
            resolved.to_string(),
            "file:///tmp/job-artifacts/downloads/data.json"
        );
    }

    #[test]
    fn test_sandboxed_path_rejects_escapes() {
        let root = sandbox_root();
        for path in [
            "",
            "/etc/passwd",
            "../escape.json",
            "a/../../escape.json",
            "~/x.json",
            "file:///etc/passwd",
            "gs://bucket/x.json",
            "a/./b.json",
            " padded.json",
        ] {
            assert!(
                resolve_sandboxed_path(&root, path).is_err(),
                "should be rejected: {path:?}"
            );
        }
    }
}
