use base64::{engine::general_purpose, Engine as _};
use bytes::Bytes;
use std::sync::Arc;

use reearth_flow_eval_expr::engine::Engine as ExprEngine;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{Attribute, AttributeValue};

use super::client::HttpResponse;
use super::errors::{HttpProcessorError, Result};
use super::params::{ResponseEncoding, ResponseHandling};

/// Process and store HTTP response
pub(crate) fn process_response(
    response: HttpResponse,
    handling: &Option<ResponseHandling>,
    encoding: &Option<ResponseEncoding>,
    auto_detect: bool,
    max_size: Option<u64>,
    engine: &Arc<ExprEngine>,
    scope: &reearth_flow_eval_expr::scope::Scope,
    storage_resolver: &Arc<StorageResolver>,
    attributes: &mut indexmap::IndexMap<Attribute, AttributeValue>,
    response_body_attr: &str,
    status_code_attr: &str,
    headers_attr: &str,
) -> Result<()> {
    // Check response size limit
    if let Some(max_size) = max_size {
        let body_size = response.body.len() as u64;
        if body_size > max_size {
            return Err(HttpProcessorError::Response(format!(
                "Response body size ({} bytes) exceeds maximum allowed size ({} bytes)",
                body_size, max_size
            )));
        }
    }

    // Add status code
    attributes.insert(
        Attribute::new(status_code_attr.to_string()),
        AttributeValue::Number(response.status_code.into()),
    );

    // Add headers as JSON string
    if let Ok(headers_json) = serde_json::to_string(&response.headers) {
        attributes.insert(
            Attribute::new(headers_attr.to_string()),
            AttributeValue::String(headers_json),
        );
    }

    // Determine encoding
    let effective_encoding = if auto_detect {
        detect_encoding_from_headers(&response.headers).or(encoding.clone())
    } else {
        encoding.clone()
    }
    .unwrap_or(ResponseEncoding::Text);

    // Handle response based on configuration
    match handling.as_ref().unwrap_or(&ResponseHandling::Attribute) {
        ResponseHandling::Attribute => {
            // Store in attribute (default behavior)
            let encoded_body = encode_response_body(&response.body, &effective_encoding);
            attributes.insert(
                Attribute::new(response_body_attr.to_string()),
                AttributeValue::String(encoded_body),
            );
        }
        ResponseHandling::File {
            path,
            store_path_in_attribute,
            path_attribute,
        } => {
            // Evaluate output path
            let path_ast = engine.compile(path.as_ref()).map_err(|e| {
                HttpProcessorError::Request(format!("Failed to compile output path expression: {e:?}"))
            })?;

            let output_path = scope.eval_ast::<String>(&path_ast).map_err(|e| {
                HttpProcessorError::Response(format!("Failed to evaluate output path: {e:?}"))
            })?;

            // Save response to file
            save_response_to_file(&response.body, &output_path, storage_resolver)?;

            // Optionally store path in attribute
            if store_path_in_attribute.unwrap_or(true) {
                let attr_name = path_attribute
                    .clone()
                    .unwrap_or_else(|| "_response_file_path".to_string());
                attributes.insert(
                    Attribute::new(attr_name),
                    AttributeValue::String(output_path.clone()),
                );
            }

            // Also store metadata in response body attribute
            let metadata = serde_json::json!({
                "saved_to_file": true,
                "file_path": output_path,
                "size_bytes": response.body.len(),
            });
            attributes.insert(
                Attribute::new(response_body_attr.to_string()),
                AttributeValue::String(metadata.to_string()),
            );
        }
    }

    Ok(())
}

/// Encode response body according to encoding type
fn encode_response_body(body: &str, encoding: &ResponseEncoding) -> String {
    match encoding {
        ResponseEncoding::Text => body.to_string(),
        ResponseEncoding::Base64 => general_purpose::STANDARD.encode(body.as_bytes()),
        ResponseEncoding::Binary => {
            // Store as base64 for binary data to preserve in JSON
            general_purpose::STANDARD.encode(body.as_bytes())
        }
    }
}

/// Detect encoding from Content-Type header
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

/// Save response body to file using storage
fn save_response_to_file(
    body: &str,
    path: &str,
    storage_resolver: &Arc<StorageResolver>,
) -> Result<()> {
    let uri = reearth_flow_common::uri::Uri::for_test(path);
    let storage = storage_resolver
        .resolve(&uri)
        .map_err(|e| HttpProcessorError::Response(format!("Failed to resolve storage path '{}': {}", path, e)))?;

    let path_string = uri.path().as_path().display().to_string();
    let storage_path = std::path::Path::new(&path_string);

    let bytes = Bytes::from(body.as_bytes().to_vec());

    storage
        .put_sync(storage_path, bytes)
        .map_err(|e| {
            HttpProcessorError::Response(format!("Failed to save response to file '{}': {}", path, e))
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_text() {
        let body = "Hello World";
        let encoded = encode_response_body(body, &ResponseEncoding::Text);
        assert_eq!(encoded, "Hello World");
    }

    #[test]
    fn test_encode_base64() {
        let body = "Hello";
        let encoded = encode_response_body(body, &ResponseEncoding::Base64);
        assert_eq!(encoded, "SGVsbG8=");
    }

    #[test]
    fn test_detect_json_encoding() {
        let mut headers = std::collections::HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let encoding = detect_encoding_from_headers(&headers);
        assert!(matches!(encoding, Some(ResponseEncoding::Text)));
    }

    #[test]
    fn test_detect_image_encoding() {
        let mut headers = std::collections::HashMap::new();
        headers.insert("Content-Type".to_string(), "image/png".to_string());

        let encoding = detect_encoding_from_headers(&headers);
        assert!(matches!(encoding, Some(ResponseEncoding::Base64)));
    }

    #[test]
    fn test_detect_pdf_encoding() {
        let mut headers = std::collections::HashMap::new();
        headers.insert("content-type".to_string(), "application/pdf".to_string());

        let encoding = detect_encoding_from_headers(&headers);
        assert!(matches!(encoding, Some(ResponseEncoding::Base64)));
    }

    #[test]
    fn test_size_limit_check() {
        let response = HttpResponse {
            status_code: 200,
            headers: std::collections::HashMap::new(),
            body: "a".repeat(1000),
        };

        let engine = Arc::new(ExprEngine::new());
        let scope = engine.new_scope();
        let storage_resolver = Arc::new(StorageResolver::new());
        let mut attributes = indexmap::IndexMap::new();

        // Should fail with size limit
        let result = process_response(
            response.clone(),
            &None,
            &None,
            true,
            Some(500), // Max 500 bytes
            &engine,
            &scope,
            &storage_resolver,
            &mut attributes,
            "_response",
            "_status",
            "_headers",
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
    }

    #[test]
    fn test_response_attribute_storage() {
        let response = HttpResponse {
            status_code: 200,
            headers: std::collections::HashMap::from([(
                "content-type".to_string(),
                "application/json".to_string(),
            )]),
            body: r#"{"result": "success"}"#.to_string(),
        };

        let engine = Arc::new(ExprEngine::new());
        let scope = engine.new_scope();
        let storage_resolver = Arc::new(StorageResolver::new());
        let mut attributes = indexmap::IndexMap::new();

        let result = process_response(
            response,
            &None,
            &Some(ResponseEncoding::Text),
            false,
            None,
            &engine,
            &scope,
            &storage_resolver,
            &mut attributes,
            "_response",
            "_status",
            "_headers",
        );

        assert!(result.is_ok());
        assert_eq!(attributes.len(), 3); // status, headers, body
        assert!(attributes.contains_key(&Attribute::new("_status".to_string())));
        assert!(attributes.contains_key(&Attribute::new("_response".to_string())));
    }
}

