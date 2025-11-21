use base64::{engine::general_purpose, Engine as _};
use reqwest::blocking::multipart::{Form, Part};
use std::sync::Arc;

use reearth_flow_eval_expr::engine::Engine as ExprEngine;
use reearth_flow_storage::resolve::StorageResolver;

use super::errors::{HttpProcessorError, Result};
use super::params::{BinarySource, FormField, MultipartPart, RequestBody};

/// Built request body with content type
pub(crate) struct BuiltBody {
    pub content: BodyContent,
    pub content_type: Option<String>,
}

/// Body content types
pub(crate) enum BodyContent {
    Text(String),
    Binary(Vec<u8>),
    Form(Vec<(String, String)>),
    Multipart(Form),
}

/// Build request body from configuration
pub(crate) fn build_request_body(
    body: &RequestBody,
    engine: &Arc<ExprEngine>,
    scope: &reearth_flow_eval_expr::scope::Scope,
    storage_resolver: &Arc<StorageResolver>,
) -> Result<BuiltBody> {
    match body {
        RequestBody::Text {
            content,
            content_type,
        } => {
            let content_ast = engine.compile(content.as_ref()).map_err(|e| {
                HttpProcessorError::CallerFactory(format!(
                    "Failed to compile body content expression: {e:?}"
                ))
            })?;

            let text = scope.eval_ast::<String>(&content_ast).map_err(|e| {
                HttpProcessorError::Request(format!("Failed to evaluate body content: {e:?}"))
            })?;

            Ok(BuiltBody {
                content: BodyContent::Text(text),
                content_type: content_type.clone(),
            })
        }

        RequestBody::Binary {
            source,
            content_type,
        } => {
            let binary_data = load_binary_source(source, engine, scope, storage_resolver)?;

            Ok(BuiltBody {
                content: BodyContent::Binary(binary_data),
                content_type: content_type
                    .clone()
                    .or_else(|| Some("application/octet-stream".to_string())),
            })
        }

        RequestBody::FormUrlEncoded { fields } => {
            let mut form_fields = Vec::new();

            for field in fields {
                let value_ast = engine.compile(field.value.as_ref()).map_err(|e| {
                    HttpProcessorError::CallerFactory(format!(
                        "Failed to compile form field '{}' expression: {e:?}",
                        field.name
                    ))
                })?;

                let value = scope.eval_ast::<String>(&value_ast).map_err(|e| {
                    HttpProcessorError::Request(format!(
                        "Failed to evaluate form field '{}': {e:?}",
                        field.name
                    ))
                })?;

                form_fields.push((field.name.clone(), value));
            }

            Ok(BuiltBody {
                content: BodyContent::Form(form_fields),
                content_type: Some("application/x-www-form-urlencoded".to_string()),
            })
        }

        RequestBody::Multipart { parts } => {
            let mut form = Form::new();

            for part in parts {
                form = add_multipart_part(form, part, engine, scope, storage_resolver)?;
            }

            Ok(BuiltBody {
                content: BodyContent::Multipart(form),
                content_type: None, // reqwest sets this automatically with boundary
            })
        }
    }
}

/// Load binary data from source
fn load_binary_source(
    source: &BinarySource,
    engine: &Arc<ExprEngine>,
    scope: &reearth_flow_eval_expr::scope::Scope,
    storage_resolver: &Arc<StorageResolver>,
) -> Result<Vec<u8>> {
    match source {
        BinarySource::Base64 { data } => {
            let data_ast = engine.compile(data.as_ref()).map_err(|e| {
                HttpProcessorError::CallerFactory(format!(
                    "Failed to compile base64 data expression: {e:?}"
                ))
            })?;

            let base64_str = scope.eval_ast::<String>(&data_ast).map_err(|e| {
                HttpProcessorError::Request(format!("Failed to evaluate base64 data: {e:?}"))
            })?;

            general_purpose::STANDARD
                .decode(base64_str.as_bytes())
                .map_err(|e| {
                    HttpProcessorError::Request(format!("Failed to decode base64 data: {e}"))
                })
        }

        BinarySource::File { path } => {
            let path_ast = engine.compile(path.as_ref()).map_err(|e| {
                HttpProcessorError::CallerFactory(format!(
                    "Failed to compile file path expression: {e:?}"
                ))
            })?;

            let file_path_str = scope.eval_ast::<String>(&path_ast).map_err(|e| {
                HttpProcessorError::Request(format!("Failed to evaluate file path: {e:?}"))
            })?;

            // Parse and resolve storage path
            let uri = reearth_flow_common::uri::Uri::for_test(&file_path_str);
            let storage = storage_resolver
                .resolve(&uri)
                .map_err(|e| HttpProcessorError::Request(format!("Failed to resolve storage path '{}': {}", file_path_str, e)))?;

            // Get path from URI
            let path_string = uri.path().as_path().display().to_string();
            let storage_path = std::path::Path::new(&path_string);

            let bytes = storage
                .get_sync(storage_path)
                .map_err(|e| {
                    HttpProcessorError::Request(format!(
                        "Failed to read file '{}': {}",
                        file_path_str, e
                    ))
                })?;

            Ok(bytes.to_vec())
        }
    }
}

/// Add a part to multipart form
fn add_multipart_part(
    form: Form,
    part: &MultipartPart,
    engine: &Arc<ExprEngine>,
    scope: &reearth_flow_eval_expr::scope::Scope,
    storage_resolver: &Arc<StorageResolver>,
) -> Result<Form> {
    match part {
        MultipartPart::Text { name, value } => {
            let value_ast = engine.compile(value.as_ref()).map_err(|e| {
                HttpProcessorError::CallerFactory(format!(
                    "Failed to compile multipart text field '{}' expression: {e:?}",
                    name
                ))
            })?;

            let text_value = scope.eval_ast::<String>(&value_ast).map_err(|e| {
                HttpProcessorError::Request(format!(
                    "Failed to evaluate multipart text field '{}': {e:?}",
                    name
                ))
            })?;

            Ok(form.text(name.clone(), text_value))
        }

        MultipartPart::File {
            name,
            source,
            filename,
            content_type,
        } => {
            let file_data = load_binary_source(source, engine, scope, storage_resolver)?;

            let mut part = Part::bytes(file_data);

            if let Some(fname) = filename {
                part = part.file_name(fname.clone());
            }

            if let Some(ct) = content_type {
                part = part.mime_str(ct).map_err(|e| {
                    HttpProcessorError::Request(format!("Invalid MIME type '{}': {e}", ct))
                })?;
            }

            Ok(form.part(name.clone(), part))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_types::Expr;

    #[test]
    fn test_text_body() {
        let engine = Arc::new(ExprEngine::new());
        let scope = engine.new_scope();
        scope.set("message", "Hello World".into());

        let body = RequestBody::Text {
            content: Expr::new(r#"env.get("message")"#),
            content_type: Some("text/plain".to_string()),
        };

        let storage_resolver = Arc::new(StorageResolver::new());
        let result = build_request_body(&body, &engine, &scope, &storage_resolver);

        assert!(result.is_ok());
        let built = result.unwrap();
        assert_eq!(built.content_type, Some("text/plain".to_string()));
        match built.content {
            BodyContent::Text(text) => assert_eq!(text, "Hello World"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_base64_binary_body() {
        let engine = Arc::new(ExprEngine::new());
        let scope = engine.new_scope();
        // "Hello" in base64
        scope.set("data", "SGVsbG8=".into());

        let body = RequestBody::Binary {
            source: BinarySource::Base64 {
                data: Expr::new(r#"env.get("data")"#),
            },
            content_type: Some("application/octet-stream".to_string()),
        };

        let storage_resolver = Arc::new(StorageResolver::new());
        let result = build_request_body(&body, &engine, &scope, &storage_resolver);

        assert!(result.is_ok());
        let built = result.unwrap();
        match built.content {
            BodyContent::Binary(data) => assert_eq!(data, b"Hello"),
            _ => panic!("Expected binary content"),
        }
    }

    #[test]
    fn test_form_urlencoded_body() {
        let engine = Arc::new(ExprEngine::new());
        let scope = engine.new_scope();
        scope.set("user", "john".into());
        scope.set("pass", "secret".into());

        let body = RequestBody::FormUrlEncoded {
            fields: vec![
                FormField {
                    name: "username".to_string(),
                    value: Expr::new(r#"env.get("user")"#),
                },
                FormField {
                    name: "password".to_string(),
                    value: Expr::new(r#"env.get("pass")"#),
                },
            ],
        };

        let storage_resolver = Arc::new(StorageResolver::new());
        let result = build_request_body(&body, &engine, &scope, &storage_resolver);

        assert!(result.is_ok());
        let built = result.unwrap();
        assert_eq!(
            built.content_type,
            Some("application/x-www-form-urlencoded".to_string())
        );
        match built.content {
            BodyContent::Form(fields) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "username");
                assert_eq!(fields[0].1, "john");
                assert_eq!(fields[1].0, "password");
                assert_eq!(fields[1].1, "secret");
            }
            _ => panic!("Expected form content"),
        }
    }

    #[test]
    fn test_multipart_text() {
        let engine = Arc::new(ExprEngine::new());
        let scope = engine.new_scope();
        scope.set("name", "John Doe".into());

        let body = RequestBody::Multipart {
            parts: vec![MultipartPart::Text {
                name: "username".to_string(),
                value: Expr::new(r#"env.get("name")"#),
            }],
        };

        let storage_resolver = Arc::new(StorageResolver::new());
        let result = build_request_body(&body, &engine, &scope, &storage_resolver);

        assert!(result.is_ok());
        // Can't easily test Form internals, but we verify it doesn't error
    }
}

