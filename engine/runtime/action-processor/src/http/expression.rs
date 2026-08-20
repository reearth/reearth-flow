use reearth_flow_types::{Code, CompiledCode};

use super::errors::{HttpProcessorError, Result};
use super::params::{
    ApiKeyLocation, Authentication, BinarySource, HeaderParam, MultipartPart, QueryParam,
    RequestBody, ResponseHandling,
};

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

#[derive(Debug, Clone)]
pub(crate) enum CompiledAuthentication {
    Basic {
        username_ast: CompiledCode,
        password_ast: CompiledCode,
    },
    Bearer {
        token_ast: CompiledCode,
    },
    ApiKey {
        key_name: String,
        key_value_ast: CompiledCode,
        location: ApiKeyLocation,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum CompiledBinarySource {
    Base64 { data_ast: CompiledCode },
    File { path_ast: CompiledCode },
}

#[derive(Debug, Clone)]
pub(crate) struct CompiledFormField {
    pub name: String,
    pub value_ast: CompiledCode,
}

#[derive(Debug, Clone)]
pub(crate) enum CompiledMultipartPart {
    Text {
        name: String,
        value_ast: CompiledCode,
    },
    File {
        name: String,
        source: CompiledBinarySource,
        filename: Option<String>,
        content_type: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum CompiledRequestBody {
    Text {
        content_ast: CompiledCode,
        content_type: Option<String>,
    },
    Binary {
        source: CompiledBinarySource,
        content_type: Option<String>,
    },
    FormUrlEncoded {
        fields: Vec<CompiledFormField>,
    },
    Multipart {
        parts: Vec<CompiledMultipartPart>,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum CompiledResponseHandling {
    Attribute,
    File {
        path_ast: CompiledCode,
        store_path_in_attribute: Option<bool>,
        path_attribute: Option<String>,
    },
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

    pub fn compile_auth(&self, auth: &Authentication) -> Result<CompiledAuthentication> {
        match auth {
            Authentication::Basic { username, password } => Ok(CompiledAuthentication::Basic {
                username_ast: username.compile().map_err(|e| {
                    HttpProcessorError::CallerFactory(format!(
                        "Failed to compile username expression: {e:?}"
                    ))
                })?,
                password_ast: password.compile().map_err(|e| {
                    HttpProcessorError::CallerFactory(format!(
                        "Failed to compile password expression: {e:?}"
                    ))
                })?,
            }),
            Authentication::Bearer { token } => Ok(CompiledAuthentication::Bearer {
                token_ast: token.compile().map_err(|e| {
                    HttpProcessorError::CallerFactory(format!(
                        "Failed to compile token expression: {e:?}"
                    ))
                })?,
            }),
            Authentication::ApiKey {
                key_name,
                key_value,
                location,
            } => Ok(CompiledAuthentication::ApiKey {
                key_name: key_name.clone(),
                key_value_ast: key_value.compile().map_err(|e| {
                    HttpProcessorError::CallerFactory(format!(
                        "Failed to compile API key expression: {e:?}"
                    ))
                })?,
                location: location.clone(),
            }),
        }
    }

    fn compile_binary_source(&self, source: &BinarySource) -> Result<CompiledBinarySource> {
        match source {
            BinarySource::Base64 { data } => Ok(CompiledBinarySource::Base64 {
                data_ast: data.compile().map_err(|e| {
                    HttpProcessorError::CallerFactory(format!(
                        "Failed to compile base64 data expression: {e:?}"
                    ))
                })?,
            }),
            BinarySource::File { path } => Ok(CompiledBinarySource::File {
                path_ast: path.compile().map_err(|e| {
                    HttpProcessorError::CallerFactory(format!(
                        "Failed to compile file path expression: {e:?}"
                    ))
                })?,
            }),
        }
    }

    fn compile_multipart_part(&self, part: &MultipartPart) -> Result<CompiledMultipartPart> {
        match part {
            MultipartPart::Text { name, value } => Ok(CompiledMultipartPart::Text {
                name: name.clone(),
                value_ast: value.compile().map_err(|e| {
                    HttpProcessorError::CallerFactory(format!(
                        "Failed to compile multipart text field '{name}' expression: {e:?}"
                    ))
                })?,
            }),
            MultipartPart::File {
                name,
                source,
                filename,
                content_type,
            } => Ok(CompiledMultipartPart::File {
                name: name.clone(),
                source: self.compile_binary_source(source)?,
                filename: filename.clone(),
                content_type: content_type.clone(),
            }),
        }
    }

    pub fn compile_body(&self, body: &RequestBody) -> Result<CompiledRequestBody> {
        match body {
            RequestBody::Text {
                content,
                content_type,
            } => Ok(CompiledRequestBody::Text {
                content_ast: content.compile().map_err(|e| {
                    HttpProcessorError::CallerFactory(format!(
                        "Failed to compile body content expression: {e:?}"
                    ))
                })?,
                content_type: content_type.clone(),
            }),
            RequestBody::Binary {
                source,
                content_type,
            } => Ok(CompiledRequestBody::Binary {
                source: self.compile_binary_source(source)?,
                content_type: content_type.clone(),
            }),
            RequestBody::FormUrlEncoded { fields } => Ok(CompiledRequestBody::FormUrlEncoded {
                fields: fields
                    .iter()
                    .map(|f| {
                        Ok(CompiledFormField {
                            name: f.name.clone(),
                            value_ast: f.value.compile().map_err(|e| {
                                HttpProcessorError::CallerFactory(format!(
                                    "Failed to compile form field '{}' expression: {e:?}",
                                    f.name
                                ))
                            })?,
                        })
                    })
                    .collect::<Result<_>>()?,
            }),
            RequestBody::Multipart { parts } => Ok(CompiledRequestBody::Multipart {
                parts: parts
                    .iter()
                    .map(|p| self.compile_multipart_part(p))
                    .collect::<Result<_>>()?,
            }),
        }
    }

    pub fn compile_response_handling(
        &self,
        handling: &ResponseHandling,
    ) -> Result<CompiledResponseHandling> {
        match handling {
            ResponseHandling::Attribute => Ok(CompiledResponseHandling::Attribute),
            ResponseHandling::File {
                path,
                store_path_in_attribute,
                path_attribute,
            } => Ok(CompiledResponseHandling::File {
                path_ast: path.compile().map_err(|e| {
                    HttpProcessorError::CallerFactory(format!(
                        "Failed to compile response path expression: {e:?}"
                    ))
                })?,
                store_path_in_attribute: *store_path_in_attribute,
                path_attribute: path_attribute.clone(),
            }),
        }
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
