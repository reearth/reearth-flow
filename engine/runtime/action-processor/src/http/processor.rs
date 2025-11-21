use std::{collections::HashMap, sync::Arc};

use reearth_flow_runtime::{
    errors::BoxedError,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Processor, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue};
use serde_json::Value;

use super::client::{HttpClient, HttpResponse};
use super::expression::{CompiledHeader, CompiledQueryParam};
use super::params::HttpCallerParam;
use super::request::RequestBuilder;

/// HTTP Caller processor that executes HTTP requests and enriches features
#[derive(Clone)]
pub struct HttpCallerProcessor {
    global_params: Option<HashMap<String, Value>>,
    client: Arc<dyn HttpClient>,
    params: HttpCallerParam,
    url_ast: rhai::AST,
    compiled_headers: Vec<CompiledHeader>,
    compiled_query_params: Vec<CompiledQueryParam>,
}

impl std::fmt::Debug for HttpCallerProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpCallerProcessor")
            .field("params", &self.params)
            .finish()
    }
}

impl HttpCallerProcessor {
    /// Create a new HttpCallerProcessor instance
    pub fn new(
        global_params: Option<HashMap<String, Value>>,
        client: Arc<dyn HttpClient>,
        params: HttpCallerParam,
        url_ast: rhai::AST,
        compiled_headers: Vec<CompiledHeader>,
        compiled_query_params: Vec<CompiledQueryParam>,
    ) -> Self {
        Self {
            global_params,
            client,
            params,
            url_ast,
            compiled_headers,
            compiled_query_params,
        }
    }

    #[cfg(test)]
    pub fn with_client(
        client: Arc<dyn HttpClient>,
        params: HttpCallerParam,
        url_ast: rhai::AST,
    ) -> Self {
        Self {
            global_params: None,
            client,
            params,
            url_ast,
            compiled_headers: Vec::new(),
            compiled_query_params: Vec::new(),
        }
    }

    /// Send HTTP request and enrich feature with response
    fn execute_request(
        &self,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let feature = &ctx.feature;
        let scope = feature.new_scope(expr_engine.clone(), &self.global_params);

        // Evaluate URL
        let url = match scope.eval_ast::<String>(&self.url_ast) {
            Ok(url) => url,
            Err(e) => {
                let error_msg = format!("Failed to evaluate URL expression: {e:?}");
                self.send_rejected_feature(ctx, fw, &error_msg);
                return Ok(());
            }
        };

        // Build request
        let method = self.params.method.clone().into();
        let builder = RequestBuilder::new(method, url);

        // Build request body if configured
        let built_body = if let Some(body_config) = &self.params.request_body {
            match super::body::build_request_body(
                body_config,
                &expr_engine,
                &scope,
                &ctx.storage_resolver,
            ) {
                Ok(body) => Some(body),
                Err(e) => {
                    self.send_rejected_feature(ctx, fw, &e.to_string());
                    return Ok(());
                }
            }
        } else {
            None
        };

        let builder = match builder
            .with_headers(&self.compiled_headers, &scope)
            .and_then(|b| {
                b.with_content_type(
                    built_body
                        .as_ref()
                        .and_then(|b| b.content_type.as_deref())
                        .or(self.params.content_type.as_deref()),
                )
            })
            .and_then(|b| b.with_query_params(&self.compiled_query_params, &scope))
            .and_then(|b| b.with_body(built_body.map(|b| b.content)))
        {
            Ok(builder) => builder,
            Err(e) => {
                self.send_rejected_feature(ctx, fw, &e.to_string());
                return Ok(());
            }
        };

        let (method, url, mut headers, mut query_params, body) = builder.build();

        // Apply authentication if configured
        if let Some(auth) = &self.params.authentication {
            if let Err(e) = super::auth::apply_authentication(auth, &expr_engine, &scope, &mut headers, &mut query_params) {
                self.send_rejected_feature(ctx, fw, &e.to_string());
                return Ok(());
            }
        }

        // Send HTTP request
        match self
            .client
            .send_request(method, &url, headers, query_params, body)
        {
            Ok(response) => {
                self.handle_success_response(ctx, fw, response);
            }
            Err(e) => {
                let error_msg = format!("HTTP request failed: {e}");
                self.send_rejected_feature(ctx, fw, &error_msg);
            }
        }

        Ok(())
    }

    /// Handle successful HTTP response
    fn handle_success_response(
        &self,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
        response: HttpResponse,
    ) {
        let mut new_feature = ctx.feature.clone();

        // Add response body
        new_feature.attributes.insert(
            Attribute::new(self.params.response_body_attribute.clone()),
            AttributeValue::String(response.body),
        );

        // Add status code
        new_feature.attributes.insert(
            Attribute::new(self.params.status_code_attribute.clone()),
            AttributeValue::Number(response.status_code.into()),
        );

        // Add headers as JSON string
        if let Ok(headers_json) = serde_json::to_string(&response.headers) {
            new_feature.attributes.insert(
                Attribute::new(self.params.headers_attribute.clone()),
                AttributeValue::String(headers_json),
            );
        }

        fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
    }

    /// Send feature to rejected port with error message
    fn send_rejected_feature(
        &self,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
        error_msg: &str,
    ) {
        ctx.event_hub
            .error_log(Some(ctx.error_span()), error_msg.to_string());

        let mut rejected_feature = ctx.feature.clone();
        rejected_feature.attributes.insert(
            Attribute::new(self.params.error_attribute.clone()),
            AttributeValue::String(error_msg.to_string()),
        );

        fw.send(ctx.new_with_feature_and_port(
            rejected_feature,
            REJECTED_PORT.clone(),
        ));
    }
}

impl Processor for HttpCallerProcessor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        self.execute_request(&ctx, fw)
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "HTTPCaller"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::client::MockHttpClient;
    use crate::http::params::HttpMethod;
    use reearth_flow_eval_expr::engine::Engine;
    use reearth_flow_types::Expr;

    #[test]
    fn test_processor_creation() {
        let mock_response = HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: r#"{"result": "success"}"#.to_string(),
        };

        let mock_client =
            MockHttpClient::new().with_response("https://example.com/test", Ok(mock_response));

        let params = HttpCallerParam {
            url: Expr::new("https://example.com/test"),
            method: HttpMethod::Get,
            custom_headers: None,
            query_parameters: None,
            request_body: None,
            content_type: None,
            response_body_attribute: "_response_body".to_string(),
            status_code_attribute: "_http_status_code".to_string(),
            headers_attribute: "_headers".to_string(),
            error_attribute: "_http_error".to_string(),
            connection_timeout: Some(60),
            transfer_timeout: Some(90),
            authentication: None,
            user_agent: None,
            verify_ssl: None,
            follow_redirects: None,
            max_redirects: None,
        };

        let engine = Engine::new();
        let url_ast = engine.compile(r#""https://example.com/test""#).unwrap();

        let processor = HttpCallerProcessor::with_client(Arc::new(mock_client), params, url_ast);

        assert_eq!(processor.name(), "HTTPCaller");
    }

    #[test]
    fn test_processor_debug() {
        let mock_client = MockHttpClient::new();
        let params = HttpCallerParam {
            url: Expr::new("https://example.com"),
            method: HttpMethod::Get,
            custom_headers: None,
            query_parameters: None,
            request_body: None,
            content_type: None,
            response_body_attribute: "_response_body".to_string(),
            status_code_attribute: "_http_status_code".to_string(),
            headers_attribute: "_headers".to_string(),
            error_attribute: "_http_error".to_string(),
            connection_timeout: None,
            transfer_timeout: None,
            authentication: None,
            user_agent: None,
            verify_ssl: None,
            follow_redirects: None,
            max_redirects: None,
        };

        let engine = Engine::new();
        let url_ast = engine.compile(r#""https://example.com""#).unwrap();

        let processor = HttpCallerProcessor::with_client(Arc::new(mock_client), params, url_ast);

        let debug_str = format!("{:?}", processor);
        assert!(debug_str.contains("HttpCallerProcessor"));
    }
}

