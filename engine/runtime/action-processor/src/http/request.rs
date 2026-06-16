use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Method;
use std::sync::Arc;

use super::body::BodyContent;
use super::errors::{HttpProcessorError, Result};
use super::expression::{CompiledHeader, CompiledQueryParam};

pub(crate) struct RequestBuilder {
    method: Method,
    url: String,
    headers: HeaderMap,
    query_params: Vec<(String, String)>,
    body: Option<BodyContent>,
}

impl RequestBuilder {
    pub fn new(method: Method, url: String) -> Self {
        Self {
            method,
            url,
            headers: HeaderMap::new(),
            query_params: Vec::new(),
            body: None,
        }
    }

    pub fn with_headers(
        mut self,
        compiled_headers: &[CompiledHeader],
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<Self> {
        for compiled_header in compiled_headers {
            let value = compiled_header
                .value_ast
                .eval_string_env_only(env_vars.clone())
                .map_err(|e| {
                    HttpProcessorError::Request(format!(
                        "Failed to evaluate header '{}': {e:?}",
                        compiled_header.name
                    ))
                })?;

            let header_name =
                HeaderName::from_bytes(compiled_header.name.as_bytes()).map_err(|e| {
                    HttpProcessorError::Request(format!(
                        "Invalid header name '{}': {e}",
                        compiled_header.name
                    ))
                })?;

            let header_value = HeaderValue::from_str(&value).map_err(|e| {
                HttpProcessorError::Request(format!(
                    "Invalid header value for '{}': {e}",
                    compiled_header.name
                ))
            })?;

            self.headers.insert(header_name, header_value);
        }
        Ok(self)
    }

    pub fn with_content_type(mut self, content_type: Option<&str>) -> Result<Self> {
        if let Some(content_type) = content_type {
            let value = HeaderValue::from_str(content_type).map_err(|e| {
                HttpProcessorError::Request(format!("Invalid Content-Type value: {e}"))
            })?;
            self.headers.insert(reqwest::header::CONTENT_TYPE, value);
        }
        Ok(self)
    }

    pub fn with_query_params(
        mut self,
        compiled_params: &[CompiledQueryParam],
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<Self> {
        for compiled_param in compiled_params {
            let value = compiled_param
                .value_ast
                .eval_string_env_only(env_vars.clone())
                .map_err(|e| {
                    HttpProcessorError::Request(format!(
                        "Failed to evaluate query parameter '{}': {e:?}",
                        compiled_param.name
                    ))
                })?;
            self.query_params.push((compiled_param.name.clone(), value));
        }
        Ok(self)
    }

    pub fn with_body(mut self, body: Option<BodyContent>) -> Result<Self> {
        self.body = body;
        Ok(self)
    }

    pub fn build(
        self,
    ) -> (
        Method,
        String,
        HeaderMap,
        Vec<(String, String)>,
        Option<BodyContent>,
    ) {
        (
            self.method,
            self.url,
            self.headers,
            self.query_params,
            self.body,
        )
    }
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
    fn test_request_builder_evaluates_query_params_from_scope() {
        let env_vars = make_env(&[("id", "123")]);

        let compiled_params = vec![CompiledQueryParam {
            name: "user_id".to_string(),
            value_ast: Code {
                ty: CodeType::FlowExpr,
                value: r#"env["id"]"#.to_string(),
            }
            .compile()
            .unwrap(),
        }];

        let builder = RequestBuilder::new(Method::GET, "https://example.com".to_string());
        let result = builder.with_query_params(&compiled_params, env_vars);

        assert!(result.is_ok());
        let builder = result.unwrap();
        let (_method, _url, _headers, query, _body) = builder.build();

        assert_eq!(query[0].1, "123");
    }
}
