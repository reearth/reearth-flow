//! HTTP types and parameter definitions for HTTPCaller

use reearth_flow_types::Expr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// HTTP methods supported by the HTTPCaller
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
#[allow(clippy::upper_case_acronyms)]
pub enum HttpMethod {
    #[default]
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl HttpMethod {
    /// Convert to reqwest HTTP method
    pub fn to_reqwest_method(&self) -> reqwest::Method {
        match self {
            HttpMethod::GET => reqwest::Method::GET,
            HttpMethod::POST => reqwest::Method::POST,
            HttpMethod::PUT => reqwest::Method::PUT,
            HttpMethod::DELETE => reqwest::Method::DELETE,
            HttpMethod::PATCH => reqwest::Method::PATCH,
            HttpMethod::HEAD => reqwest::Method::HEAD,
            HttpMethod::OPTIONS => reqwest::Method::OPTIONS,
        }
    }

    /// Check if the method supports a request body
    pub fn supports_body(&self) -> bool {
        matches!(self, HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH)
    }
}

/// A name/value pair for headers or query parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NameValuePair {
    /// # Name
    /// The name/key of the header or query parameter
    pub name: Expr,
    /// # Value
    /// The value of the header or query parameter (supports expressions)
    pub value: Expr,
}

/// # HTTPCaller Parameters
/// Configure HTTP request parameters for making requests based on feature data
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HttpCallerParam {
    /// # Request URL
    /// The URL to send the HTTP request to. Supports expressions to dynamically construct URLs from feature attributes.
    /// Example: `"https://api.example.com/data/" + env.get("__value").id`
    pub request_url: Expr,

    /// # HTTP Method
    /// The HTTP method to use for the request (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS).
    /// Defaults to GET if not specified.
    #[serde(default)]
    pub http_method: Option<HttpMethod>,

    /// # Custom Headers
    /// List of custom headers to include in the request. Each header has a name and value,
    /// both of which support expressions for dynamic values.
    #[serde(default)]
    pub headers: Option<Vec<NameValuePair>>,

    /// # Query Parameters
    /// List of query parameters to append to the URL. Each parameter has a name and value,
    /// both of which support expressions for dynamic values.
    #[serde(default)]
    pub query_parameters: Option<Vec<NameValuePair>>,

    /// # Request Body
    /// The body content to send with POST, PUT, or PATCH requests.
    /// Supports expressions for dynamic content generation.
    #[serde(default)]
    pub request_body: Option<Expr>,

    /// # Content-Type Header
    /// The Content-Type header value for the request body.
    /// Common values: "application/json", "application/xml", "text/plain"
    #[serde(default)]
    pub content_type: Option<String>,

    /// # Response Body Attribute
    /// The name of the attribute to store the response body in.
    /// Defaults to "_response_body" if not specified.
    #[serde(default)]
    pub response_body_attribute: Option<String>,

    /// # Status Code Attribute
    /// The name of the attribute to store the HTTP status code in.
    /// Defaults to "_http_status_code" if not specified.
    #[serde(default)]
    pub status_code_attribute: Option<String>,

    /// # Headers Attribute
    /// The name of the attribute to store the response headers in.
    /// Defaults to "_headers" if not specified.
    #[serde(default)]
    pub headers_attribute: Option<String>,

    /// # Error Attribute
    /// The name of the attribute to store error messages in when a request fails.
    /// Defaults to "_http_error" if not specified.
    #[serde(default)]
    pub error_attribute: Option<String>,

    /// # Connection Timeout (seconds)
    /// Maximum time in seconds to wait for establishing a connection.
    /// Defaults to 60 seconds if not specified.
    #[serde(default)]
    pub connection_timeout_seconds: Option<u64>,

    /// # Transfer Timeout (seconds)
    /// Maximum time in seconds to wait for the complete response.
    /// Defaults to 90 seconds if not specified.
    #[serde(default)]
    pub transfer_timeout_seconds: Option<u64>,
}

/// Default attribute names for HTTP response data
pub mod defaults {
    pub const RESPONSE_BODY_ATTRIBUTE: &str = "_response_body";
    pub const STATUS_CODE_ATTRIBUTE: &str = "_http_status_code";
    pub const HEADERS_ATTRIBUTE: &str = "_headers";
    pub const ERROR_ATTRIBUTE: &str = "_http_error";
    pub const CONNECTION_TIMEOUT_SECS: u64 = 60;
    pub const TRANSFER_TIMEOUT_SECS: u64 = 90;
}

/// HTTP response data returned from a request
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub body: String,
    pub status_code: u16,
    pub headers: std::collections::HashMap<String, String>,
}

/// HTTP request configuration
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub url: String,
    pub method: HttpMethod,
    pub headers: Vec<(String, String)>,
    pub query_params: Vec<(String, String)>,
    pub body: Option<String>,
    pub content_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_to_reqwest() {
        assert_eq!(HttpMethod::GET.to_reqwest_method(), reqwest::Method::GET);
        assert_eq!(HttpMethod::POST.to_reqwest_method(), reqwest::Method::POST);
        assert_eq!(HttpMethod::PUT.to_reqwest_method(), reqwest::Method::PUT);
        assert_eq!(
            HttpMethod::DELETE.to_reqwest_method(),
            reqwest::Method::DELETE
        );
        assert_eq!(
            HttpMethod::PATCH.to_reqwest_method(),
            reqwest::Method::PATCH
        );
        assert_eq!(HttpMethod::HEAD.to_reqwest_method(), reqwest::Method::HEAD);
        assert_eq!(
            HttpMethod::OPTIONS.to_reqwest_method(),
            reqwest::Method::OPTIONS
        );
    }

    #[test]
    fn test_http_method_supports_body() {
        assert!(!HttpMethod::GET.supports_body());
        assert!(HttpMethod::POST.supports_body());
        assert!(HttpMethod::PUT.supports_body());
        assert!(!HttpMethod::DELETE.supports_body());
        assert!(HttpMethod::PATCH.supports_body());
        assert!(!HttpMethod::HEAD.supports_body());
        assert!(!HttpMethod::OPTIONS.supports_body());
    }

    #[test]
    fn test_http_method_default() {
        let method: HttpMethod = Default::default();
        assert_eq!(method, HttpMethod::GET);
    }

    #[test]
    fn test_param_deserialization() {
        let json = r#"{
            "requestUrl": "\"https://api.example.com/data\"",
            "httpMethod": "POST",
            "contentType": "application/json",
            "requestBody": "\"{}\"",
            "headers": [
                {"name": "\"Authorization\"", "value": "\"Bearer token123\""}
            ],
            "queryParameters": [
                {"name": "\"page\"", "value": "\"1\""}
            ],
            "responseBodyAttribute": "_body",
            "statusCodeAttribute": "_status",
            "headersAttribute": "_resp_headers",
            "errorAttribute": "_error",
            "connectionTimeoutSeconds": 30,
            "transferTimeoutSeconds": 60
        }"#;

        let params: HttpCallerParam = serde_json::from_str(json).unwrap();
        assert_eq!(
            params.request_url.as_ref(),
            "\"https://api.example.com/data\""
        );
        assert!(matches!(params.http_method, Some(HttpMethod::POST)));
        assert_eq!(params.content_type, Some("application/json".to_string()));
        assert!(params.headers.is_some());
        assert_eq!(params.headers.as_ref().unwrap().len(), 1);
        assert!(params.query_parameters.is_some());
        assert_eq!(params.query_parameters.as_ref().unwrap().len(), 1);
        assert_eq!(params.response_body_attribute, Some("_body".to_string()));
        assert_eq!(params.status_code_attribute, Some("_status".to_string()));
        assert_eq!(params.headers_attribute, Some("_resp_headers".to_string()));
        assert_eq!(params.error_attribute, Some("_error".to_string()));
        assert_eq!(params.connection_timeout_seconds, Some(30));
        assert_eq!(params.transfer_timeout_seconds, Some(60));
    }

    #[test]
    fn test_param_defaults() {
        let json = r#"{
            "requestUrl": "\"https://api.example.com\""
        }"#;

        let params: HttpCallerParam = serde_json::from_str(json).unwrap();
        assert!(params.http_method.is_none());
        assert!(params.headers.is_none());
        assert!(params.query_parameters.is_none());
        assert!(params.request_body.is_none());
        assert!(params.content_type.is_none());
        assert!(params.response_body_attribute.is_none());
        assert!(params.status_code_attribute.is_none());
        assert!(params.headers_attribute.is_none());
        assert!(params.error_attribute.is_none());
        assert!(params.connection_timeout_seconds.is_none());
        assert!(params.transfer_timeout_seconds.is_none());
    }
}
