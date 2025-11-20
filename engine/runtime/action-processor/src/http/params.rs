use reearth_flow_types::Expr;
use reqwest::Method;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// # HTTP Caller Parameters
/// Configure HTTP/HTTPS requests with dynamic values from feature attributes
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HttpCallerParam {
    /// # Request URL
    /// The URL to send the request to. Supports expressions to reference feature attributes (e.g., "https://api.example.com/data/${id}")
    pub url: Expr,

    /// # HTTP Method
    /// The HTTP method to use for the request
    #[serde(default = "default_method")]
    pub method: HttpMethod,

    /// # Custom Headers
    /// List of custom HTTP headers to include in the request. Values support expressions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_headers: Option<Vec<HeaderParam>>,

    /// # Query Parameters
    /// URL query parameters to append to the request. Values support expressions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_parameters: Option<Vec<QueryParam>>,

    /// # Request Body
    /// The request body content (for POST, PUT, PATCH methods). Supports expressions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<Expr>,

    /// # Content-Type Header
    /// The Content-Type header value for the request body
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    /// # Response Body Attribute Name
    /// Name of the attribute to store the response body in
    #[serde(default = "default_response_body_attr")]
    pub response_body_attribute: String,

    /// # Status Code Attribute Name
    /// Name of the attribute to store the HTTP status code in
    #[serde(default = "default_status_code_attr")]
    pub status_code_attribute: String,

    /// # Headers Attribute Name
    /// Name of the attribute to store response headers in (as JSON string)
    #[serde(default = "default_headers_attr")]
    pub headers_attribute: String,

    /// # Error Attribute Name
    /// Name of the attribute to store error messages in (for rejected features)
    #[serde(default = "default_error_attr")]
    pub error_attribute: String,

    /// # Connection Timeout (seconds)
    /// Maximum time to wait for connection establishment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_timeout: Option<u64>,

    /// # Transfer Timeout (seconds)
    /// Maximum time to wait for the entire request/response cycle
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transfer_timeout: Option<u64>,

    /// # Authentication
    /// Authentication method to use for the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<Authentication>,

    /// # User-Agent
    /// Custom User-Agent header value (default: "reearth-flow-http-caller/1.0")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    /// # Verify SSL/TLS Certificates
    /// Whether to verify HTTPS certificates (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_ssl: Option<bool>,

    /// # Follow Redirects
    /// Whether to automatically follow HTTP redirects (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_redirects: Option<bool>,

    /// # Maximum Redirects
    /// Maximum number of redirects to follow (default: 10)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_redirects: Option<u8>,
}

fn default_method() -> HttpMethod {
    HttpMethod::Get
}

fn default_response_body_attr() -> String {
    "_response_body".to_string()
}

fn default_status_code_attr() -> String {
    "_http_status_code".to_string()
}

fn default_headers_attr() -> String {
    "_headers".to_string()
}

fn default_error_attr() -> String {
    "_http_error".to_string()
}

/// HTTP methods supported by the HTTPCaller
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
    Copy,
    Lock,
    Mkcol,
    Move,
    Propfind,
    Proppatch,
    Unlock,
}

impl From<HttpMethod> for Method {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::Get => Method::GET,
            HttpMethod::Post => Method::POST,
            HttpMethod::Put => Method::PUT,
            HttpMethod::Delete => Method::DELETE,
            HttpMethod::Patch => Method::PATCH,
            HttpMethod::Head => Method::HEAD,
            HttpMethod::Options => Method::OPTIONS,
            HttpMethod::Copy => Method::from_bytes(b"COPY").unwrap(),
            HttpMethod::Lock => Method::from_bytes(b"LOCK").unwrap(),
            HttpMethod::Mkcol => Method::from_bytes(b"MKCOL").unwrap(),
            HttpMethod::Move => Method::from_bytes(b"MOVE").unwrap(),
            HttpMethod::Propfind => Method::from_bytes(b"PROPFIND").unwrap(),
            HttpMethod::Proppatch => Method::from_bytes(b"PROPPATCH").unwrap(),
            HttpMethod::Unlock => Method::from_bytes(b"UNLOCK").unwrap(),
        }
    }
}

/// Custom HTTP header parameter
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HeaderParam {
    /// # Header Name
    /// The name of the HTTP header
    pub name: String,

    /// # Header Value
    /// The value of the HTTP header. Supports expressions.
    pub value: Expr,
}

/// URL query parameter
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct QueryParam {
    /// # Parameter Name
    /// The name of the query parameter
    pub name: String,

    /// # Parameter Value
    /// The value of the query parameter. Supports expressions.
    pub value: Expr,
}

/// Authentication methods supported by HTTPCaller
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Authentication {
    /// # Basic Authentication
    /// Username and password authentication
    #[serde(rename_all = "camelCase")]
    Basic {
        /// # Username
        /// Username for basic authentication. Supports expressions.
        username: Expr,
        /// # Password
        /// Password for basic authentication. Supports expressions.
        password: Expr,
    },
    /// # Bearer Token Authentication
    /// Token-based authentication (e.g., OAuth 2.0)
    #[serde(rename_all = "camelCase")]
    Bearer {
        /// # Token
        /// Bearer token value. Supports expressions.
        token: Expr,
    },
    /// # API Key Authentication
    /// API key in header or query parameter
    #[serde(rename_all = "camelCase")]
    ApiKey {
        /// # Key Name
        /// Name of the header or query parameter
        key_name: String,
        /// # Key Value
        /// API key value. Supports expressions.
        key_value: Expr,
        /// # Location
        /// Where to send the API key (header or query)
        #[serde(default = "default_api_key_location")]
        location: ApiKeyLocation,
    },
}

/// Location for API key authentication
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ApiKeyLocation {
    /// Send API key as HTTP header
    Header,
    /// Send API key as query parameter
    Query,
}

fn default_api_key_location() -> ApiKeyLocation {
    ApiKeyLocation::Header
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_conversion() {
        assert_eq!(Method::from(HttpMethod::Get), Method::GET);
        assert_eq!(Method::from(HttpMethod::Post), Method::POST);
        assert_eq!(Method::from(HttpMethod::Put), Method::PUT);
        assert_eq!(Method::from(HttpMethod::Delete), Method::DELETE);
        assert_eq!(Method::from(HttpMethod::Patch), Method::PATCH);
        assert_eq!(Method::from(HttpMethod::Head), Method::HEAD);
        assert_eq!(Method::from(HttpMethod::Options), Method::OPTIONS);
        assert_eq!(
            Method::from(HttpMethod::Copy),
            Method::from_bytes(b"COPY").unwrap()
        );
        assert_eq!(
            Method::from(HttpMethod::Lock),
            Method::from_bytes(b"LOCK").unwrap()
        );
        assert_eq!(
            Method::from(HttpMethod::Mkcol),
            Method::from_bytes(b"MKCOL").unwrap()
        );
        assert_eq!(
            Method::from(HttpMethod::Move),
            Method::from_bytes(b"MOVE").unwrap()
        );
        assert_eq!(
            Method::from(HttpMethod::Propfind),
            Method::from_bytes(b"PROPFIND").unwrap()
        );
        assert_eq!(
            Method::from(HttpMethod::Proppatch),
            Method::from_bytes(b"PROPPATCH").unwrap()
        );
        assert_eq!(
            Method::from(HttpMethod::Unlock),
            Method::from_bytes(b"UNLOCK").unwrap()
        );
    }

    #[test]
    fn test_default_attribute_names() {
        assert_eq!(default_response_body_attr(), "_response_body");
        assert_eq!(default_status_code_attr(), "_http_status_code");
        assert_eq!(default_headers_attr(), "_headers");
        assert_eq!(default_error_attr(), "_http_error");
    }

    #[test]
    fn test_default_method() {
        let method = default_method();
        assert!(matches!(method, HttpMethod::Get));
    }

    #[test]
    fn test_param_serialization() {
        let params = HttpCallerParam {
            url: Expr::new("https://example.com"),
            method: HttpMethod::Post,
            custom_headers: Some(vec![HeaderParam {
                name: "Authorization".to_string(),
                value: Expr::new("Bearer ${token}"),
            }]),
            query_parameters: Some(vec![QueryParam {
                name: "id".to_string(),
                value: Expr::new("${feature_id}"),
            }]),
            request_body: Some(Expr::new(r#"{"data": "${value}"}"#)),
            content_type: Some("application/json".to_string()),
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

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("https://example.com"));
        assert!(json.contains("POST"));
    }

    #[test]
    fn test_basic_auth() {
        let auth = Authentication::Basic {
            username: Expr::new("user123"),
            password: Expr::new("pass456"),
        };
        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("basic"));
        assert!(json.contains("user123"));
    }

    #[test]
    fn test_bearer_auth() {
        let auth = Authentication::Bearer {
            token: Expr::new("token123"),
        };
        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("bearer"));
        assert!(json.contains("token123"));
    }

    #[test]
    fn test_api_key_auth() {
        let auth = Authentication::ApiKey {
            key_name: "X-API-Key".to_string(),
            key_value: Expr::new("key123"),
            location: ApiKeyLocation::Header,
        };
        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("apiKey"));
        assert!(json.contains("X-API-Key"));
    }
}
