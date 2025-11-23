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
use super::metrics::{RequestMetrics, RequestTimer};
use super::params::HttpCallerParam;
use super::rate_limit::RateLimiter;
use super::request::RequestBuilder;

pub struct HttpCallerProcessor {
    global_params: Option<HashMap<String, Value>>,
    client: Arc<dyn HttpClient>,
    params: HttpCallerParam,
    url_ast: rhai::AST,
    compiled_headers: Vec<CompiledHeader>,
    compiled_query_params: Vec<CompiledQueryParam>,
    rate_limiter: Option<Arc<RateLimiter>>,
}

impl Clone for HttpCallerProcessor {
    fn clone(&self) -> Self {
        Self {
            global_params: self.global_params.clone(),
            client: self.client.clone(),
            params: self.params.clone(),
            url_ast: self.url_ast.clone(),
            compiled_headers: self.compiled_headers.clone(),
            compiled_query_params: self.compiled_query_params.clone(),
            rate_limiter: self.rate_limiter.clone(),
        }
    }
}

impl std::fmt::Debug for HttpCallerProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpCallerProcessor")
            .field("params", &self.params)
            .finish()
    }
}

impl HttpCallerProcessor {
    pub fn new(
        global_params: Option<HashMap<String, Value>>,
        client: Arc<dyn HttpClient>,
        params: HttpCallerParam,
        url_ast: rhai::AST,
        compiled_headers: Vec<CompiledHeader>,
        compiled_query_params: Vec<CompiledQueryParam>,
    ) -> Self {
        let rate_limiter = params
            .rate_limit
            .as_ref()
            .map(|config| Arc::new(RateLimiter::new(config.clone())));

        Self {
            global_params,
            client,
            params,
            url_ast,
            compiled_headers,
            compiled_query_params,
            rate_limiter,
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
            rate_limiter: None,
        }
    }

    fn execute_request(
        &self,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let feature = &ctx.feature;
        let scope = feature.new_scope(expr_engine.clone(), &self.global_params);

        let url = match scope.eval_ast::<String>(&self.url_ast) {
            Ok(url) => url,
            Err(e) => {
                let error_msg = format!("Failed to evaluate URL expression: {e:?}");
                self.send_rejected_feature(ctx, fw, &error_msg);
                return Ok(());
            }
        };

        let method = self.params.method.clone().into();
        let builder = RequestBuilder::new(method, url);

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

        if let Some(auth) = &self.params.authentication {
            if let Err(e) = super::auth::apply_authentication(
                auth,
                &expr_engine,
                &scope,
                &mut headers,
                &mut query_params,
            ) {
                self.send_rejected_feature(ctx, fw, &e.to_string());
                return Ok(());
            }
        }

        if let Some(rate_limiter) = &self.rate_limiter {
            rate_limiter.acquire();
        }

        let timer = RequestTimer::new();

        let result = super::retry::execute_with_retry(
            self.client.as_ref(),
            method,
            url,
            headers,
            query_params,
            body,
            &self.params.retry,
        );

        match result {
            Ok((response, retry_ctx)) => {
                let metrics = RequestMetrics::new(timer.elapsed(), &response, &retry_ctx);
                self.handle_success_response(ctx, fw, response, metrics);
            }
            Err(e) => {
                let error_msg = format!("HTTP request failed after retries: {e}");
                ctx.event_hub
                    .error_log(Some(ctx.error_span()), error_msg.clone());
                self.send_rejected_feature(ctx, fw, &error_msg);
            }
        }

        Ok(())
    }

    fn handle_success_response(
        &self,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
        response: HttpResponse,
        metrics: RequestMetrics,
    ) {
        let mut new_feature = ctx.feature.clone();
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = new_feature.new_scope(expr_engine.clone(), &self.global_params);

        let config = super::response::ResponseProcessorConfig {
            handling: &self.params.response_handling,
            encoding: &self.params.response_encoding,
            auto_detect: self.params.auto_detect_encoding.unwrap_or(true),
            max_size: self.params.max_response_size,
            engine: &expr_engine,
            scope: &scope,
            storage_resolver: &ctx.storage_resolver,
            response_body_attr: &self.params.response_body_attribute,
            status_code_attr: &self.params.status_code_attribute,
            headers_attr: &self.params.headers_attribute,
        };

        let result =
            super::response::process_response(response, &config, &mut new_feature.attributes);

        match result {
            Ok(()) => {
                metrics.add_to_attributes(&mut new_feature.attributes, &self.params.observability);

                fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
            }
            Err(e) => {
                let error_msg = format!("Failed to process response: {e}");
                ctx.event_hub
                    .error_log(Some(ctx.error_span()), error_msg.clone());
                self.send_rejected_feature(ctx, fw, &error_msg);
            }
        }
    }

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

        fw.send(ctx.new_with_feature_and_port(rejected_feature, REJECTED_PORT.clone()));
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
            response_handling: None,
            max_response_size: None,
            response_encoding: None,
            auto_detect_encoding: None,
            retry: None,
            rate_limit: None,
            observability: None,
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
            response_handling: None,
            max_response_size: None,
            response_encoding: None,
            auto_detect_encoding: None,
            retry: None,
            rate_limit: None,
            observability: None,
        };

        let engine = Engine::new();
        let url_ast = engine.compile(r#""https://example.com""#).unwrap();

        let processor = HttpCallerProcessor::with_client(Arc::new(mock_client), params, url_ast);

        let debug_str = format!("{processor:?}");
        assert!(debug_str.contains("HttpCallerProcessor"));
    }
}
