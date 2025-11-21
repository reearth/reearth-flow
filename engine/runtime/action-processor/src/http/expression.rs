use std::sync::Arc;

use reearth_flow_eval_expr::engine::Engine;

use super::errors::{HttpProcessorError, Result};
use super::params::{HeaderParam, QueryParam};

/// Compiled header with evaluated AST
#[derive(Debug, Clone)]
pub(crate) struct CompiledHeader {
    pub name: String,
    pub value_ast: rhai::AST,
}

/// Compiled query parameter with evaluated AST
#[derive(Debug, Clone)]
pub(crate) struct CompiledQueryParam {
    pub name: String,
    pub value_ast: rhai::AST,
}

/// Expression compiler for HTTP request components
pub(crate) struct ExpressionCompiler {
    engine: Arc<Engine>,
}

impl ExpressionCompiler {
    pub fn new(engine: Arc<Engine>) -> Self {
        Self { engine }
    }

    /// Compile URL expression
    pub fn compile_url(&self, url_expr: &str) -> Result<rhai::AST> {
        self.engine.compile(url_expr).map_err(|e| {
            HttpProcessorError::CallerFactory(format!("Failed to compile URL expression: {e:?}"))
        })
    }

    /// Compile custom headers expressions
    pub fn compile_headers(&self, headers: &[HeaderParam]) -> Result<Vec<CompiledHeader>> {
        let mut compiled = Vec::new();
        for header in headers {
            let value_ast = self.engine.compile(header.value.as_ref()).map_err(|e| {
                HttpProcessorError::CallerFactory(format!(
                    "Failed to compile header '{}' value expression: {e:?}",
                    header.name
                ))
            })?;
            compiled.push(CompiledHeader {
                name: header.name.clone(),
                value_ast,
            });
        }
        Ok(compiled)
    }

    /// Compile query parameters expressions
    pub fn compile_query_params(&self, params: &[QueryParam]) -> Result<Vec<CompiledQueryParam>> {
        let mut compiled = Vec::new();
        for param in params {
            let value_ast = self.engine.compile(param.value.as_ref()).map_err(|e| {
                HttpProcessorError::CallerFactory(format!(
                    "Failed to compile query parameter '{}' expression: {e:?}",
                    param.name
                ))
            })?;
            compiled.push(CompiledQueryParam {
                name: param.name.clone(),
                value_ast,
            });
        }
        Ok(compiled)
    }

    /// Compile request body expression
    pub fn compile_body(&self, body_expr: &str) -> Result<rhai::AST> {
        self.engine.compile(body_expr).map_err(|e| {
            HttpProcessorError::CallerFactory(format!(
                "Failed to compile request body expression: {e:?}"
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_types::Expr;

    #[test]
    fn test_expression_compiler() {
        let engine = Arc::new(Engine::new());
        let compiler = ExpressionCompiler::new(engine);

        // Test URL compilation
        let url_ast = compiler.compile_url(r#""https://example.com/test""#);
        assert!(url_ast.is_ok());

        // Test header compilation
        let headers = vec![HeaderParam {
            name: "Authorization".to_string(),
            value: Expr::new(r#""Bearer token""#),
        }];
        let compiled_headers = compiler.compile_headers(&headers);
        assert!(compiled_headers.is_ok());
        assert_eq!(compiled_headers.unwrap().len(), 1);

        // Test query param compilation
        let params = vec![QueryParam {
            name: "id".to_string(),
            value: Expr::new(r#""123""#),
        }];
        let compiled_params = compiler.compile_query_params(&params);
        assert!(compiled_params.is_ok());
        assert_eq!(compiled_params.unwrap().len(), 1);

        // Test body compilation
        let body_ast = compiler.compile_body(r#"`{"test": "value"}`"#);
        assert!(body_ast.is_ok());
    }
}
