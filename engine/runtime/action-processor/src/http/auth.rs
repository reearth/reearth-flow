use base64::{engine::general_purpose, Engine as _};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION};
use std::sync::Arc;

use super::errors::{HttpProcessorError, Result};
use super::params::{ApiKeyLocation, Authentication};
use reearth_flow_eval_expr::engine::Engine as ExprEngine;

pub(crate) fn apply_authentication(
    auth: &Authentication,
    engine: &Arc<ExprEngine>,
    scope: &reearth_flow_eval_expr::scope::Scope,
    headers: &mut HeaderMap,
    query_params: &mut Vec<(String, String)>,
) -> Result<()> {
    match auth {
        Authentication::Basic { username, password } => {
            let username_ast = engine.compile(username.as_ref()).map_err(|e| {
                HttpProcessorError::CallerFactory(format!(
                    "Failed to compile username expression: {e:?}"
                ))
            })?;

            let username_val = scope.eval_ast::<String>(&username_ast).map_err(|e| {
                HttpProcessorError::Request(format!("Failed to evaluate username: {e:?}"))
            })?;

            let password_ast = engine.compile(password.as_ref()).map_err(|e| {
                HttpProcessorError::CallerFactory(format!(
                    "Failed to compile password expression: {e:?}"
                ))
            })?;

            let password_val = scope.eval_ast::<String>(&password_ast).map_err(|e| {
                HttpProcessorError::Request(format!("Failed to evaluate password: {e:?}"))
            })?;

            let credentials = format!("{}:{}", username_val, password_val);
            let encoded = general_purpose::STANDARD.encode(credentials.as_bytes());
            let auth_value = format!("Basic {}", encoded);

            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&auth_value).map_err(|e| {
                    HttpProcessorError::Request(format!("Invalid basic auth header value: {e}"))
                })?,
            );
        }
        Authentication::Bearer { token } => {
            let token_ast = engine.compile(token.as_ref()).map_err(|e| {
                HttpProcessorError::CallerFactory(format!(
                    "Failed to compile token expression: {e:?}"
                ))
            })?;

            let token_val = scope.eval_ast::<String>(&token_ast).map_err(|e| {
                HttpProcessorError::Request(format!("Failed to evaluate token: {e:?}"))
            })?;

            let auth_value = format!("Bearer {}", token_val);
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
            let key_ast = engine.compile(key_value.as_ref()).map_err(|e| {
                HttpProcessorError::CallerFactory(format!(
                    "Failed to compile API key expression: {e:?}"
                ))
            })?;

            let key_val = scope.eval_ast::<String>(&key_ast).map_err(|e| {
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
    use reearth_flow_eval_expr::engine::Engine;
    use reearth_flow_types::Expr;

    #[test]
    fn test_basic_auth() {
        let engine = Arc::new(Engine::new());
        let scope = engine.new_scope();
        scope.set("username", "testuser".into());
        scope.set("password", "testpass".into());

        let auth = Authentication::Basic {
            username: Expr::new(r#"env.get("username")"#),
            password: Expr::new(r#"env.get("password")"#),
        };

        let mut headers = HeaderMap::new();
        let mut query_params = Vec::new();

        let result = apply_authentication(&auth, &engine, &scope, &mut headers, &mut query_params);
        assert!(result.is_ok());
        assert!(headers.contains_key(AUTHORIZATION));

        let auth_header = headers.get(AUTHORIZATION).unwrap().to_str().unwrap();
        assert!(auth_header.starts_with("Basic "));
    }

    #[test]
    fn test_bearer_auth() {
        let engine = Arc::new(Engine::new());
        let scope = engine.new_scope();
        scope.set("token", "abc123".into());

        let auth = Authentication::Bearer {
            token: Expr::new(r#"env.get("token")"#),
        };

        let mut headers = HeaderMap::new();
        let mut query_params = Vec::new();

        let result = apply_authentication(&auth, &engine, &scope, &mut headers, &mut query_params);
        assert!(result.is_ok());
        assert!(headers.contains_key(AUTHORIZATION));

        let auth_header = headers.get(AUTHORIZATION).unwrap().to_str().unwrap();
        assert_eq!(auth_header, "Bearer abc123");
    }

    #[test]
    fn test_api_key_header() {
        let engine = Arc::new(Engine::new());
        let scope = engine.new_scope();
        scope.set("api_key", "key123".into());

        let auth = Authentication::ApiKey {
            key_name: "X-API-Key".to_string(),
            key_value: Expr::new(r#"env.get("api_key")"#),
            location: ApiKeyLocation::Header,
        };

        let mut headers = HeaderMap::new();
        let mut query_params = Vec::new();

        let result = apply_authentication(&auth, &engine, &scope, &mut headers, &mut query_params);
        assert!(result.is_ok());
        assert!(headers.contains_key("x-api-key"));

        let key_header = headers.get("x-api-key").unwrap().to_str().unwrap();
        assert_eq!(key_header, "key123");
    }

    #[test]
    fn test_api_key_query() {
        let engine = Arc::new(Engine::new());
        let scope = engine.new_scope();
        scope.set("api_key", "key456".into());

        let auth = Authentication::ApiKey {
            key_name: "apikey".to_string(),
            key_value: Expr::new(r#"env.get("api_key")"#),
            location: ApiKeyLocation::Query,
        };

        let mut headers = HeaderMap::new();
        let mut query_params = Vec::new();

        let result = apply_authentication(&auth, &engine, &scope, &mut headers, &mut query_params);
        assert!(result.is_ok());
        assert_eq!(query_params.len(), 1);
        assert_eq!(query_params[0].0, "apikey");
        assert_eq!(query_params[0].1, "key456");
    }
}
