use base64::{engine::general_purpose, Engine as _};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION};
use std::sync::Arc;

use super::errors::{HttpProcessorError, Result};
use super::params::{ApiKeyLocation, Authentication};

pub(crate) fn apply_authentication(
    auth: &Authentication,
    env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    headers: &mut HeaderMap,
    query_params: &mut Vec<(String, String)>,
) -> Result<()> {
    match auth {
        Authentication::Basic { username, password } => {
            let username_val = username
                .compile()
                .map_err(|e| {
                    HttpProcessorError::CallerFactory(format!(
                        "Failed to compile username expression: {e:?}"
                    ))
                })?
                .eval_string_env_only(env_vars.clone())
                .map_err(|e| {
                    HttpProcessorError::Request(format!("Failed to evaluate username: {e:?}"))
                })?;

            let password_val = password
                .compile()
                .map_err(|e| {
                    HttpProcessorError::CallerFactory(format!(
                        "Failed to compile password expression: {e:?}"
                    ))
                })?
                .eval_string_env_only(env_vars.clone())
                .map_err(|e| {
                    HttpProcessorError::Request(format!("Failed to evaluate password: {e:?}"))
                })?;

            let credentials = format!("{username_val}:{password_val}");
            let encoded = general_purpose::STANDARD.encode(credentials.as_bytes());
            let auth_value = format!("Basic {encoded}");

            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&auth_value).map_err(|e| {
                    HttpProcessorError::Request(format!("Invalid basic auth header value: {e}"))
                })?,
            );
        }
        Authentication::Bearer { token } => {
            let token_val = token
                .compile()
                .map_err(|e| {
                    HttpProcessorError::CallerFactory(format!(
                        "Failed to compile token expression: {e:?}"
                    ))
                })?
                .eval_string_env_only(env_vars.clone())
                .map_err(|e| {
                    HttpProcessorError::Request(format!("Failed to evaluate token: {e:?}"))
                })?;

            let auth_value = format!("Bearer {token_val}");
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&auth_value).map_err(|e| {
                    HttpProcessorError::Request(format!("Invalid bearer token header value: {e}"))
                })?,
            );
        }
        Authentication::ApiKey {
            key_name,
            key_value,
            location,
        } => {
            let key_val = key_value
                .compile()
                .map_err(|e| {
                    HttpProcessorError::CallerFactory(format!(
                        "Failed to compile API key expression: {e:?}"
                    ))
                })?
                .eval_string_env_only(env_vars.clone())
                .map_err(|e| {
                    HttpProcessorError::Request(format!("Failed to evaluate API key: {e:?}"))
                })?;

            match location {
                ApiKeyLocation::Header => {
                    let header_name = HeaderName::from_bytes(key_name.as_bytes()).map_err(|e| {
                        HttpProcessorError::Request(format!("Invalid API key header name: {e}"))
                    })?;

                    let header_value = HeaderValue::from_str(&key_val).map_err(|e| {
                        HttpProcessorError::Request(format!("Invalid API key header value: {e}"))
                    })?;

                    headers.insert(header_name, header_value);
                }
                ApiKeyLocation::Query => {
                    query_params.push((key_name.clone(), key_val));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_types::{Code, CodeType};

    fn make_env(pairs: &[(&str, &str)]) -> Arc<serde_json::Map<String, serde_json::Value>> {
        let mut map = serde_json::Map::new();
        for (k, v) in pairs {
            map.insert(k.to_string(), serde_json::Value::String(v.to_string()));
        }
        Arc::new(map)
    }

    #[test]
    fn test_basic_auth() {
        let env_vars = make_env(&[("username", "testuser"), ("password", "testpass")]);

        let auth = Authentication::Basic {
            username: Code {
                ty: CodeType::FlowExpr,
                value: r#"env["username"]"#.to_string(),
            },
            password: Code {
                ty: CodeType::FlowExpr,
                value: r#"env["password"]"#.to_string(),
            },
        };

        let mut headers = HeaderMap::new();
        let mut query_params = Vec::new();

        let result = apply_authentication(&auth, env_vars, &mut headers, &mut query_params);
        assert!(result.is_ok());
        assert!(headers.contains_key(AUTHORIZATION));

        let auth_header = headers.get(AUTHORIZATION).unwrap().to_str().unwrap();
        assert!(auth_header.starts_with("Basic "));
    }

    #[test]
    fn test_bearer_auth() {
        let env_vars = make_env(&[("token", "abc123")]);

        let auth = Authentication::Bearer {
            token: Code {
                ty: CodeType::FlowExpr,
                value: r#"env["token"]"#.to_string(),
            },
        };

        let mut headers = HeaderMap::new();
        let mut query_params = Vec::new();

        let result = apply_authentication(&auth, env_vars, &mut headers, &mut query_params);
        assert!(result.is_ok());
        assert!(headers.contains_key(AUTHORIZATION));

        let auth_header = headers.get(AUTHORIZATION).unwrap().to_str().unwrap();
        assert_eq!(auth_header, "Bearer abc123");
    }

    #[test]
    fn test_api_key_header() {
        let env_vars = make_env(&[("api_key", "key123")]);

        let auth = Authentication::ApiKey {
            key_name: "X-API-Key".to_string(),
            key_value: Code {
                ty: CodeType::FlowExpr,
                value: r#"env["api_key"]"#.to_string(),
            },
            location: ApiKeyLocation::Header,
        };

        let mut headers = HeaderMap::new();
        let mut query_params = Vec::new();

        let result = apply_authentication(&auth, env_vars, &mut headers, &mut query_params);
        assert!(result.is_ok());
        assert!(headers.contains_key("x-api-key"));

        let key_header = headers.get("x-api-key").unwrap().to_str().unwrap();
        assert_eq!(key_header, "key123");
    }

    #[test]
    fn test_api_key_query() {
        let env_vars = make_env(&[("api_key", "key456")]);

        let auth = Authentication::ApiKey {
            key_name: "apikey".to_string(),
            key_value: Code {
                ty: CodeType::FlowExpr,
                value: r#"env["api_key"]"#.to_string(),
            },
            location: ApiKeyLocation::Query,
        };

        let mut headers = HeaderMap::new();
        let mut query_params = Vec::new();

        let result = apply_authentication(&auth, env_vars, &mut headers, &mut query_params);
        assert!(result.is_ok());
        assert_eq!(query_params.len(), 1);
        assert_eq!(query_params[0].0, "apikey");
        assert_eq!(query_params[0].1, "key456");
    }
}
