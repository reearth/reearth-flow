//! HTTPCaller processor implementation

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use indexmap::IndexMap;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue};
use rhai::Dynamic;
use serde_json::Value;

use super::client::{HttpClient, HttpClientConfig, ReqwestHttpClient};
use super::errors::HttpProcessorError;
use super::types::{
    defaults, HttpCallerParam, HttpMethod, HttpRequest, HttpResponse, NameValuePair,
};

// ============================================================================
// Compiled Expression Types
// ============================================================================

/// Compiled name/value pair for efficient evaluation
#[derive(Debug, Clone)]
struct CompiledNameValuePair {
    name: rhai::AST,
    value: rhai::AST,
}

/// Compiled expressions from parameters
#[derive(Debug, Clone)]
struct CompiledExpressions {
    url: rhai::AST,
    headers: Option<Vec<CompiledNameValuePair>>,
    query_params: Option<Vec<CompiledNameValuePair>>,
    body: Option<rhai::AST>,
}

// ============================================================================
// Expression Compiler
// ============================================================================

/// Compiles parameter expressions for efficient evaluation
struct ExpressionCompiler<'a> {
    engine: &'a Arc<Engine>,
}

impl<'a> ExpressionCompiler<'a> {
    fn new(engine: &'a Arc<Engine>) -> Self {
        Self { engine }
    }

    /// Compile all expressions from parameters
    fn compile(&self, params: &HttpCallerParam) -> Result<CompiledExpressions, HttpProcessorError> {
        let url = self.compile_url(&params.request_url)?;
        let headers = self.compile_name_value_pairs(&params.headers, "header")?;
        let query_params =
            self.compile_name_value_pairs(&params.query_parameters, "query parameter")?;
        let body = self.compile_body(&params.request_body)?;

        Ok(CompiledExpressions {
            url,
            headers,
            query_params,
            body,
        })
    }

    fn compile_url(&self, url: &reearth_flow_types::Expr) -> Result<rhai::AST, HttpProcessorError> {
        self.engine.compile(url.as_ref()).map_err(|e| {
            HttpProcessorError::HttpCallerFactory(format!(
                "Failed to compile URL expression: {e:?}"
            ))
        })
    }

    fn compile_name_value_pairs(
        &self,
        pairs: &Option<Vec<NameValuePair>>,
        context: &str,
    ) -> Result<Option<Vec<CompiledNameValuePair>>, HttpProcessorError> {
        let Some(pairs) = pairs else {
            return Ok(None);
        };

        let mut compiled = Vec::with_capacity(pairs.len());
        for pair in pairs {
            let name = self.engine.compile(pair.name.as_ref()).map_err(|e| {
                HttpProcessorError::HttpCallerFactory(format!(
                    "Failed to compile {context} name expression: {e:?}"
                ))
            })?;
            let value = self.engine.compile(pair.value.as_ref()).map_err(|e| {
                HttpProcessorError::HttpCallerFactory(format!(
                    "Failed to compile {context} value expression: {e:?}"
                ))
            })?;
            compiled.push(CompiledNameValuePair { name, value });
        }
        Ok(Some(compiled))
    }

    fn compile_body(
        &self,
        body: &Option<reearth_flow_types::Expr>,
    ) -> Result<Option<rhai::AST>, HttpProcessorError> {
        let Some(body) = body else {
            return Ok(None);
        };

        let ast = self.engine.compile(body.as_ref()).map_err(|e| {
            HttpProcessorError::HttpCallerFactory(format!(
                "Failed to compile request body expression: {e:?}"
            ))
        })?;
        Ok(Some(ast))
    }
}

// ============================================================================
// Request Builder
// ============================================================================

/// Builds HTTP requests from feature data and compiled expressions
struct RequestBuilder<'a> {
    expressions: &'a CompiledExpressions,
    method: &'a HttpMethod,
    content_type: &'a Option<String>,
}

impl<'a> RequestBuilder<'a> {
    fn new(
        expressions: &'a CompiledExpressions,
        method: &'a HttpMethod,
        content_type: &'a Option<String>,
    ) -> Self {
        Self {
            expressions,
            method,
            content_type,
        }
    }

    /// Build an HTTP request from feature context
    fn build(
        &self,
        scope: &reearth_flow_eval_expr::scope::Scope,
    ) -> Result<HttpRequest, HttpProcessorError> {
        let url = self.evaluate_url(scope)?;
        let headers = self.evaluate_name_value_pairs(&self.expressions.headers, scope)?;
        let query_params = self.evaluate_name_value_pairs(&self.expressions.query_params, scope)?;
        let body = self.evaluate_body(scope)?;

        Ok(HttpRequest {
            url,
            method: self.method.clone(),
            headers,
            query_params,
            body,
            content_type: self.content_type.clone(),
        })
    }

    fn evaluate_url(
        &self,
        scope: &reearth_flow_eval_expr::scope::Scope,
    ) -> Result<String, HttpProcessorError> {
        scope
            .eval_ast::<Dynamic>(&self.expressions.url)
            .map_err(|e| {
                HttpProcessorError::Expression(format!("Failed to evaluate URL expression: {e:?}"))
            })?
            .into_string()
            .map_err(|_| {
                HttpProcessorError::Expression(
                    "URL expression must evaluate to a string".to_string(),
                )
            })
    }

    fn evaluate_name_value_pairs(
        &self,
        pairs: &Option<Vec<CompiledNameValuePair>>,
        scope: &reearth_flow_eval_expr::scope::Scope,
    ) -> Result<Vec<(String, String)>, HttpProcessorError> {
        let Some(pairs) = pairs else {
            return Ok(Vec::new());
        };

        let mut result = Vec::with_capacity(pairs.len());
        for pair in pairs {
            let name = scope
                .eval_ast::<Dynamic>(&pair.name)
                .map_err(|e| {
                    HttpProcessorError::Expression(format!(
                        "Failed to evaluate name expression: {e:?}"
                    ))
                })?
                .into_string()
                .map_err(|_| {
                    HttpProcessorError::Expression(
                        "Name expression must evaluate to a string".to_string(),
                    )
                })?;

            let value = scope
                .eval_ast::<Dynamic>(&pair.value)
                .map_err(|e| {
                    HttpProcessorError::Expression(format!(
                        "Failed to evaluate value expression: {e:?}"
                    ))
                })?
                .into_string()
                .map_err(|_| {
                    HttpProcessorError::Expression(
                        "Value expression must evaluate to a string".to_string(),
                    )
                })?;

            result.push((name, value));
        }
        Ok(result)
    }

    fn evaluate_body(
        &self,
        scope: &reearth_flow_eval_expr::scope::Scope,
    ) -> Result<Option<String>, HttpProcessorError> {
        let Some(ref body_ast) = self.expressions.body else {
            return Ok(None);
        };

        if !self.method.supports_body() {
            return Ok(None);
        }

        let body = scope
            .eval_ast::<Dynamic>(body_ast)
            .map_err(|e| {
                HttpProcessorError::Expression(format!("Failed to evaluate body expression: {e:?}"))
            })?
            .into_string()
            .map_err(|_| {
                HttpProcessorError::Expression(
                    "Body expression must evaluate to a string".to_string(),
                )
            })?;

        Ok(Some(body))
    }
}

// ============================================================================
// Response Handler
// ============================================================================

/// Configuration for response attribute names
#[derive(Debug, Clone)]
struct ResponseConfig {
    body_attribute: String,
    status_code_attribute: String,
    headers_attribute: String,
    error_attribute: String,
}

impl ResponseConfig {
    fn from_params(params: &HttpCallerParam) -> Self {
        Self {
            body_attribute: params
                .response_body_attribute
                .clone()
                .unwrap_or_else(|| defaults::RESPONSE_BODY_ATTRIBUTE.to_string()),
            status_code_attribute: params
                .status_code_attribute
                .clone()
                .unwrap_or_else(|| defaults::STATUS_CODE_ATTRIBUTE.to_string()),
            headers_attribute: params
                .headers_attribute
                .clone()
                .unwrap_or_else(|| defaults::HEADERS_ATTRIBUTE.to_string()),
            error_attribute: params
                .error_attribute
                .clone()
                .unwrap_or_else(|| defaults::ERROR_ATTRIBUTE.to_string()),
        }
    }

    /// Create attributes from a successful response
    fn success_attributes(&self, response: &HttpResponse) -> IndexMap<Attribute, AttributeValue> {
        let mut attributes = IndexMap::new();

        // Add response body
        attributes.insert(
            Attribute::new(self.body_attribute.clone()),
            AttributeValue::String(response.body.clone()),
        );

        // Add status code
        attributes.insert(
            Attribute::new(self.status_code_attribute.clone()),
            AttributeValue::Number(serde_json::Number::from(response.status_code)),
        );

        // Add headers as a map
        let headers_map: HashMap<String, AttributeValue> = response
            .headers
            .iter()
            .map(|(k, v)| (k.clone(), AttributeValue::String(v.clone())))
            .collect();
        attributes.insert(
            Attribute::new(self.headers_attribute.clone()),
            AttributeValue::Map(headers_map),
        );

        attributes
    }

    /// Create attributes from an error
    fn error_attributes(&self, error: &HttpProcessorError) -> IndexMap<Attribute, AttributeValue> {
        let mut attributes = IndexMap::new();
        attributes.insert(
            Attribute::new(self.error_attribute.clone()),
            AttributeValue::String(error.to_string()),
        );
        attributes
    }
}

// ============================================================================
// HTTPCaller Processor Factory
// ============================================================================

#[derive(Debug, Clone, Default)]
pub(super) struct HttpCallerFactory;

impl ProcessorFactory for HttpCallerFactory {
    fn name(&self) -> &str {
        "HTTPCaller"
    }

    fn description(&self) -> &str {
        "Makes HTTP/HTTPS requests based on feature data and enriches features with response information"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(HttpCallerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Web"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params = self.parse_params(&with)?;
        let expressions = self.compile_expressions(&ctx, &params)?;
        let client = self.create_client(&params);
        let response_config = ResponseConfig::from_params(&params);

        let processor = HttpCaller {
            global_params: with,
            expressions,
            method: params.http_method.unwrap_or_default(),
            content_type: params.content_type,
            response_config,
            client: Arc::new(client),
        };

        Ok(Box::new(processor))
    }
}

impl HttpCallerFactory {
    fn parse_params(
        &self,
        with: &Option<HashMap<String, Value>>,
    ) -> Result<HttpCallerParam, BoxedError> {
        let Some(with) = with.clone() else {
            return Err(HttpProcessorError::HttpCallerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let value: Value = serde_json::to_value(with).map_err(|e| {
            HttpProcessorError::HttpCallerFactory(format!(
                "Failed to serialize `with` parameter: {e}"
            ))
        })?;

        serde_json::from_value(value).map_err(|e| {
            HttpProcessorError::HttpCallerFactory(format!(
                "Failed to deserialize `with` parameter: {e}"
            ))
            .into()
        })
    }

    fn compile_expressions(
        &self,
        ctx: &NodeContext,
        params: &HttpCallerParam,
    ) -> Result<CompiledExpressions, BoxedError> {
        let compiler = ExpressionCompiler::new(&ctx.expr_engine);
        compiler.compile(params).map_err(|e| e.into())
    }

    fn create_client(&self, params: &HttpCallerParam) -> ReqwestHttpClient {
        let config = HttpClientConfig {
            connection_timeout: Duration::from_secs(
                params
                    .connection_timeout_seconds
                    .unwrap_or(defaults::CONNECTION_TIMEOUT_SECS),
            ),
            transfer_timeout: Duration::from_secs(
                params
                    .transfer_timeout_seconds
                    .unwrap_or(defaults::TRANSFER_TIMEOUT_SECS),
            ),
            ..Default::default()
        };
        ReqwestHttpClient::new(config)
    }
}

// ============================================================================
// HTTPCaller Processor
// ============================================================================

/// HTTPCaller processor that makes HTTP requests based on feature data
#[derive(Clone)]
pub struct HttpCaller {
    global_params: Option<HashMap<String, serde_json::Value>>,
    expressions: CompiledExpressions,
    method: HttpMethod,
    content_type: Option<String>,
    response_config: ResponseConfig,
    client: Arc<dyn HttpClient>,
}

impl std::fmt::Debug for HttpCaller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpCaller")
            .field("method", &self.method)
            .field("content_type", &self.content_type)
            .field("response_config", &self.response_config)
            .finish()
    }
}

impl HttpCaller {
    /// Execute the HTTP request for the given context
    fn execute_request(&self, ctx: &ExecutorContext) -> Result<HttpResponse, HttpProcessorError> {
        let feature = &ctx.feature;
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = feature.new_scope(expr_engine.clone(), &self.global_params);

        let builder = RequestBuilder::new(&self.expressions, &self.method, &self.content_type);
        let request = builder.build(&scope)?;

        self.client.execute(&request)
    }
}

impl Processor for HttpCaller {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        match self.execute_request(&ctx) {
            Ok(response) => {
                let attributes = self.response_config.success_attributes(&response);
                fw.send(ctx.new_with_feature_and_port(
                    feature.with_attributes(attributes),
                    DEFAULT_PORT.clone(),
                ));
            }
            Err(e) => {
                let attributes = self.response_config.error_attributes(&e);
                ctx.event_hub
                    .error_log(Some(ctx.error_span()), format!("HTTP request failed: {e}"));
                fw.send(ctx.new_with_feature_and_port(
                    feature.with_attributes(attributes),
                    REJECTED_PORT.clone(),
                ));
            }
        }

        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "HTTPCaller"
    }
}
