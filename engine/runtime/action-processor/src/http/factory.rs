use std::{collections::HashMap, sync::Arc};

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::NodeContext,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use serde_json::Value;

use super::client::{ClientConfig, ReqwestHttpClient};
use super::errors::{HttpProcessorError, Result};
use super::expression::ExpressionCompiler;
use super::params::HttpCallerParam;
use super::processor::HttpCallerProcessor;

/// Factory for creating HTTPCaller processor instances
#[derive(Debug, Clone, Default)]
pub struct HttpCallerFactory;

impl ProcessorFactory for HttpCallerFactory {
    fn name(&self) -> &str {
        "HTTPCaller"
    }

    fn description(&self) -> &str {
        "Make HTTP/HTTPS requests and enrich features with response data"
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
        // Parse parameters
        let params: HttpCallerParam = self.parse_parameters(with.clone())?;

        // Validate parameters
        self.validate_parameters(&params)?;

        // Create HTTP client with configuration
        let client_config = ClientConfig {
            connection_timeout: params.connection_timeout.unwrap_or(60),
            transfer_timeout: params.transfer_timeout.unwrap_or(90),
            user_agent: params.user_agent.clone(),
            verify_ssl: params.verify_ssl.unwrap_or(true),
            follow_redirects: params.follow_redirects.unwrap_or(true),
            max_redirects: params.max_redirects.unwrap_or(10),
        };
        let client = ReqwestHttpClient::with_config(client_config)?;

        // Compile expressions
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let compiler = ExpressionCompiler::new(expr_engine);

        let url_ast = compiler.compile_url(params.url.as_ref())?;

        let compiled_headers = if let Some(headers) = &params.custom_headers {
            compiler.compile_headers(headers)?
        } else {
            Vec::new()
        };

        let compiled_query_params = if let Some(query_params) = &params.query_parameters {
            compiler.compile_query_params(query_params)?
        } else {
            Vec::new()
        };

        // Create processor
        let processor = HttpCallerProcessor::new(
            with,
            Arc::new(client),
            params,
            url_ast,
            compiled_headers,
            compiled_query_params,
        );

        Ok(Box::new(processor))
    }
}

impl HttpCallerFactory {
    /// Parse and validate parameters from with clause
    fn parse_parameters(&self, with: Option<HashMap<String, Value>>) -> Result<HttpCallerParam> {
        let with = with.ok_or_else(|| {
            HttpProcessorError::CallerFactory("Missing required parameter `with`".to_string())
        })?;

        let value: Value = serde_json::to_value(with).map_err(|e| {
            HttpProcessorError::CallerFactory(format!("Failed to serialize `with` parameter: {e}"))
        })?;

        serde_json::from_value(value).map_err(|e| {
            HttpProcessorError::CallerFactory(format!(
                "Failed to deserialize `with` parameter: {e}"
            ))
        })
    }

    /// Validate required parameters
    fn validate_parameters(&self, params: &HttpCallerParam) -> Result<()> {
        if params.url.as_ref().is_empty() {
            return Err(HttpProcessorError::CallerFactory(
                "URL parameter is required".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_types::Expr;

    #[test]
    fn test_factory_name() {
        let factory = HttpCallerFactory;
        assert_eq!(factory.name(), "HTTPCaller");
    }

    #[test]
    fn test_factory_categories() {
        let factory = HttpCallerFactory;
        assert_eq!(factory.categories(), &["Web"]);
    }

    #[test]
    fn test_factory_ports() {
        let factory = HttpCallerFactory;
        assert_eq!(factory.get_input_ports().len(), 1);
        assert_eq!(factory.get_output_ports().len(), 2);
    }

    #[test]
    fn test_parse_parameters_missing_with() {
        let factory = HttpCallerFactory;
        let result = factory.parse_parameters(None);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_parameters_empty_url() {
        let factory = HttpCallerFactory;
        let params = HttpCallerParam {
            url: Expr::new(""),
            method: super::super::params::HttpMethod::Get,
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
        };

        let result = factory.validate_parameters(&params);
        assert!(result.is_err());
    }
}

