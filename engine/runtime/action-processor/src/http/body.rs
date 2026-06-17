use base64::{engine::general_purpose, Engine as _};
use reqwest::blocking::multipart::{Form, Part};
use std::sync::Arc;

use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Feature;

use super::errors::{HttpProcessorError, Result};
#[allow(unused_imports)]
use super::params::FormField;
use super::params::{BinarySource, MultipartPart, RequestBody};

pub(crate) struct BuiltBody {
    pub content: BodyContent,
    pub content_type: Option<String>,
}

pub(crate) enum BodyContent {
    Text(String),
    Binary(Vec<u8>),
    Form(Vec<(String, String)>),
    Multipart(Form),
}

pub(crate) fn build_request_body(
    body: &RequestBody,
    feature: &Feature,
    env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    storage_resolver: &Arc<StorageResolver>,
) -> Result<BuiltBody> {
    match body {
        RequestBody::Text {
            content,
            content_type,
        } => {
            let text = content
                .compile()
                .map_err(|e| {
                    HttpProcessorError::CallerFactory(format!(
                        "Failed to compile body content expression: {e:?}"
                    ))
                })?
                .eval_string(feature, env_vars.clone())
                .map_err(|e| {
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
            let binary_data = load_binary_source(source, feature, env_vars, storage_resolver)?;

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
                let value = field
                    .value
                    .compile()
                    .map_err(|e| {
                        HttpProcessorError::CallerFactory(format!(
                            "Failed to compile form field '{}' expression: {e:?}",
                            field.name
                        ))
                    })?
                    .eval_string(feature, env_vars.clone())
                    .map_err(|e| {
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
                form = add_multipart_part(form, part, feature, env_vars.clone(), storage_resolver)?;
            }

            Ok(BuiltBody {
                content: BodyContent::Multipart(form),
                content_type: None,
            })
        }
    }
}

fn load_binary_source(
    source: &BinarySource,
    feature: &Feature,
    env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    storage_resolver: &Arc<StorageResolver>,
) -> Result<Vec<u8>> {
    match source {
        BinarySource::Base64 { data } => {
            let base64_str = data
                .compile()
                .map_err(|e| {
                    HttpProcessorError::CallerFactory(format!(
                        "Failed to compile base64 data expression: {e:?}"
                    ))
                })?
                .eval_string(feature, env_vars.clone())
                .map_err(|e| {
                    HttpProcessorError::Request(format!("Failed to evaluate base64 data: {e:?}"))
                })?;

            general_purpose::STANDARD
                .decode(base64_str.as_bytes())
                .map_err(|e| {
                    HttpProcessorError::Request(format!("Failed to decode base64 data: {e}"))
                })
        }

        BinarySource::File { path } => {
            let file_path_str = path
                .compile()
                .map_err(|e| {
                    HttpProcessorError::CallerFactory(format!(
                        "Failed to compile file path expression: {e:?}"
                    ))
                })?
                .eval_string(feature, env_vars.clone())
                .map_err(|e| {
                    HttpProcessorError::Request(format!("Failed to evaluate file path: {e:?}"))
                })?;

            let uri: reearth_flow_common::uri::Uri = file_path_str.parse().map_err(|e| {
                HttpProcessorError::Request(format!(
                    "Failed to parse storage URI '{file_path_str}': {e:?}"
                ))
            })?;
            let storage = storage_resolver.resolve(&uri).map_err(|e| {
                HttpProcessorError::Request(format!(
                    "Failed to resolve storage path '{file_path_str}': {e}"
                ))
            })?;

            let path_string = uri.path().as_path().display().to_string();
            let storage_path = std::path::Path::new(&path_string);

            let bytes = storage.get_sync(storage_path).map_err(|e| {
                HttpProcessorError::Request(format!("Failed to read file '{file_path_str}': {e}"))
            })?;

            Ok(bytes.to_vec())
        }
    }
}

fn add_multipart_part(
    form: Form,
    part: &MultipartPart,
    feature: &Feature,
    env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    storage_resolver: &Arc<StorageResolver>,
) -> Result<Form> {
    match part {
        MultipartPart::Text { name, value } => {
            let text_value = value
                .compile()
                .map_err(|e| {
                    HttpProcessorError::CallerFactory(format!(
                        "Failed to compile multipart text field '{name}' expression: {e:?}"
                    ))
                })?
                .eval_string(feature, env_vars.clone())
                .map_err(|e| {
                    HttpProcessorError::Request(format!(
                        "Failed to evaluate multipart text field '{name}': {e:?}"
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
            let file_data = load_binary_source(source, feature, env_vars, storage_resolver)?;

            let mut part = Part::bytes(file_data);

            if let Some(fname) = filename {
                part = part.file_name(fname.clone());
            }

            if let Some(ct) = content_type {
                part = part.mime_str(ct).map_err(|e| {
                    HttpProcessorError::Request(format!("Invalid MIME type '{ct}': {e}"))
                })?;
            }

            Ok(form.part(name.clone(), part))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_types::{Attributes, Code, CodeType};

    fn make_env(pairs: &[(&str, &str)]) -> Arc<serde_json::Map<String, serde_json::Value>> {
        let mut map = serde_json::Map::new();
        for (k, v) in pairs {
            map.insert(k.to_string(), serde_json::Value::String(v.to_string()));
        }
        Arc::new(map)
    }

    fn empty_feature() -> Feature {
        Feature::from(Attributes::new())
    }

    #[test]
    fn test_text_body() {
        let env_vars = make_env(&[("message", "Hello World")]);

        let body = RequestBody::Text {
            content: Code {
                ty: CodeType::FlowExpr,
                value: r#"env["message"]"#.to_string(),
            },
            content_type: Some("text/plain".to_string()),
        };

        let storage_resolver = Arc::new(StorageResolver::new());
        let feature = empty_feature();
        let result = build_request_body(&body, &feature, env_vars, &storage_resolver);

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
        let env_vars = make_env(&[("data", "SGVsbG8=")]);

        let body = RequestBody::Binary {
            source: BinarySource::Base64 {
                data: Code {
                    ty: CodeType::FlowExpr,
                    value: r#"env["data"]"#.to_string(),
                },
            },
            content_type: Some("application/octet-stream".to_string()),
        };

        let storage_resolver = Arc::new(StorageResolver::new());
        let feature = empty_feature();
        let result = build_request_body(&body, &feature, env_vars, &storage_resolver);

        assert!(result.is_ok());
        let built = result.unwrap();
        match built.content {
            BodyContent::Binary(data) => assert_eq!(data, b"Hello"),
            _ => panic!("Expected binary content"),
        }
    }

    #[test]
    fn test_form_urlencoded_body() {
        let env_vars = make_env(&[("user", "john"), ("pass", "secret")]);

        let body = RequestBody::FormUrlEncoded {
            fields: vec![
                FormField {
                    name: "username".to_string(),
                    value: Code {
                        ty: CodeType::FlowExpr,
                        value: r#"env["user"]"#.to_string(),
                    },
                },
                FormField {
                    name: "password".to_string(),
                    value: Code {
                        ty: CodeType::FlowExpr,
                        value: r#"env["pass"]"#.to_string(),
                    },
                },
            ],
        };

        let storage_resolver = Arc::new(StorageResolver::new());
        let feature = empty_feature();
        let result = build_request_body(&body, &feature, env_vars, &storage_resolver);

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
        let env_vars = make_env(&[("name", "John Doe")]);

        let body = RequestBody::Multipart {
            parts: vec![MultipartPart::Text {
                name: "username".to_string(),
                value: Code {
                    ty: CodeType::FlowExpr,
                    value: r#"env["name"]"#.to_string(),
                },
            }],
        };

        let storage_resolver = Arc::new(StorageResolver::new());
        let feature = empty_feature();
        let result = build_request_body(&body, &feature, env_vars, &storage_resolver);

        assert!(result.is_ok());
    }
}
