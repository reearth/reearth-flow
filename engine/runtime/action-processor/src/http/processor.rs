use std::{collections::HashMap, sync::Arc};

use once_cell::sync::OnceCell;

use reearth_flow_runtime::{
    errors::BoxedError,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Processor, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::AttributeValue;
use serde_json::Value;

use super::client::{ClientConfig, HttpClient, HttpResponse, ReqwestHttpClient};
use super::expression::{CompiledHeader, CompiledQueryParam};
use super::metrics::{RequestMetrics, RequestTimer};
use super::params::HttpCallerParam;
use super::rate_limit::RateLimiter;
use super::request::RequestBuilder;

pub struct HttpCallerProcessor {
    global_params: Option<HashMap<String, Value>>,
    /// Lazy-initialized HTTP client. All clones of this processor share the same client
    /// for connection pooling efficiency.
    client: Arc<OnceCell<Arc<dyn HttpClient>>>,
    client_config: ClientConfig,
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
            client_config: self.client_config.clone(),
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
        client_config: ClientConfig,
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
            client: Arc::new(OnceCell::new()),
            client_config,
            params,
            url_ast,
            compiled_headers,
            compiled_query_params,
            rate_limiter,
        }
    }

    fn get_or_create_client(&self) -> Result<Arc<dyn HttpClient>, BoxedError> {
        self.client
            .get_or_try_init(|| {
                // Create client in processor's blocking thread, not in async context
                let client = ReqwestHttpClient::with_config(self.client_config.clone())?;
                Ok::<_, BoxedError>(Arc::new(client) as Arc<dyn HttpClient>)
            })
            .cloned()
    }

    #[cfg(test)]
    pub fn with_client(
        client: Arc<dyn HttpClient>,
        params: HttpCallerParam,
        url_ast: rhai::AST,
    ) -> Self {
        let cell = Arc::new(OnceCell::new());
        cell.set(client).ok();
        Self {
            global_params: None,
            client: cell,
            client_config: ClientConfig::default(),
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
        // Lazy-initialize the HTTP client in the processor's blocking thread
        let client = match self.get_or_create_client() {
            Ok(client) => client,
            Err(e) => {
                let error_msg = format!("Failed to create HTTP client: {e}");
                self.send_rejected_feature(ctx, fw, &error_msg);
                return Ok(());
            }
        };

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
            client.as_ref(),
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

        let response_config = self.params.response.as_ref();
        let handling = response_config.and_then(|r| r.response_handling.clone());
        let encoding = response_config.and_then(|r| r.response_encoding.clone());
        let config = super::response::ResponseProcessorConfig {
            handling: &handling,
            encoding: &encoding,
            auto_detect: response_config
                .and_then(|r| r.auto_detect_encoding)
                .unwrap_or(true),
            max_size: response_config.and_then(|r| r.max_response_size),
            engine: &expr_engine,
            scope: &scope,
            storage_resolver: &ctx.storage_resolver,
            response_body_attr: response_config
                .map(|r| r.response_body_attribute.as_str())
                .unwrap_or("_response_body"),
            status_code_attr: response_config
                .map(|r| r.status_code_attribute.as_str())
                .unwrap_or("_http_status_code"),
            headers_attr: response_config
                .map(|r| r.headers_attribute.as_str())
                .unwrap_or("_headers"),
        };

        let result =
            super::response::process_response(response, &config, new_feature.attributes_mut());

        match result {
            Ok(()) => {
                metrics.add_to_attributes(new_feature.attributes_mut(), &self.params.observability);

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
        let error_attr = self
            .params
            .response
            .as_ref()
            .map(|r| r.error_attribute.as_str())
            .unwrap_or("_http_error");
        rejected_feature.insert(error_attr, AttributeValue::String(error_msg.to_string()));

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

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
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
            body: r#"{"result": "success"}"#.as_bytes().to_vec(),
        };

        let mock_client =
            MockHttpClient::new().with_response("https://example.com/test", Ok(mock_response));

        let params = HttpCallerParam {
            url: Expr::new("https://example.com/test"),
            method: HttpMethod::Get,
            authentication: None,
            custom_headers: None,
            query_parameters: None,
            request_body: None,
            content_type: None,
            timeouts: Some(crate::http::params::TimeoutConfig {
                connection_timeout: Some(60),
                transfer_timeout: Some(90),
            }),
            http_options: None,
            response: None,
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
            authentication: None,
            custom_headers: None,
            query_parameters: None,
            request_body: None,
            content_type: None,
            timeouts: None,
            http_options: None,
            response: None,
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
