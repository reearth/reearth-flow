use reearth_flow_types::{Code, CompiledCode};

use super::errors::{HttpProcessorError, Result};
use super::params::{HeaderParam, QueryParam};

#[derive(Debug, Clone)]
pub(crate) struct CompiledHeader {
    pub name: String,
    pub value_ast: CompiledCode,
}

#[derive(Debug, Clone)]
pub(crate) struct CompiledQueryParam {
    pub name: String,
    pub value_ast: CompiledCode,
}

pub(crate) struct ExpressionCompiler;

impl ExpressionCompiler {
    pub fn new() -> Self {
        Self
    }

    pub fn compile_url(&self, url_expr: &Code) -> Result<CompiledCode> {
        url_expr.compile().map_err(|e| {
            HttpProcessorError::CallerFactory(format!("Failed to compile URL expression: {e:?}"))
        })
    }

    pub fn compile_headers(&self, headers: &[HeaderParam]) -> Result<Vec<CompiledHeader>> {
        let mut compiled = Vec::new();
        for header in headers {
            let value_ast = header.value.compile().map_err(|e| {
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

    pub fn compile_query_params(&self, params: &[QueryParam]) -> Result<Vec<CompiledQueryParam>> {
        let mut compiled = Vec::new();
        for param in params {
            let value_ast = param.value.compile().map_err(|e| {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_types::{Code, CodeType};

    #[test]
    fn test_expression_compiler() {
        let compiler = ExpressionCompiler::new();

        // Test URL compilation
        let url_code = Code {
            ty: CodeType::FlowExpr,
            value: r#""https://example.com/test""#.to_string(),
        };
        let url_ast = compiler.compile_url(&url_code);
        assert!(url_ast.is_ok());

        // Test header compilation
        let headers = vec![HeaderParam {
            name: "Authorization".to_string(),
            value: Code {
                ty: CodeType::FlowExpr,
                value: r#""Bearer token""#.to_string(),
            },
        }];
        let compiled_headers = compiler.compile_headers(&headers);
        assert!(compiled_headers.is_ok());
        assert_eq!(compiled_headers.unwrap().len(), 1);

        // Test query param compilation
        let params = vec![QueryParam {
            name: "id".to_string(),
            value: Code {
                ty: CodeType::FlowExpr,
                value: r#""123""#.to_string(),
            },
        }];
        let compiled_params = compiler.compile_query_params(&params);
        assert!(compiled_params.is_ok());
        assert_eq!(compiled_params.unwrap().len(), 1);
    }
}
