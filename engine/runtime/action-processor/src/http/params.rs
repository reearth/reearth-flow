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
    /// The request body configuration (text, binary, form, or multipart)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBody>,

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

    /// # Response Handling
    /// Configuration for how to handle the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_handling: Option<ResponseHandling>,

    /// # Maximum Response Size (bytes)
    /// Maximum size of response body to accept (default: unlimited)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_response_size: Option<u64>,

    /// # Response Encoding
    /// How to encode the response body
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_encoding: Option<ResponseEncoding>,

    /// # Auto Detect Encoding
    /// Automatically detect encoding from Content-Type header (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_detect_encoding: Option<bool>,

    /// # Retry Configuration
    /// Retry failed requests with exponential backoff
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,

    /// # Rate Limiting
    /// Limit request rate to avoid overwhelming servers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimitConfig>,

    /// # Observability
    /// Collect and store request metrics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observability: Option<ObservabilityConfig>,
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

/// Response handling configuration
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ResponseHandling {
    /// # Store in Attribute
    /// Store response in feature attribute (default behavior)
    #[serde(rename_all = "camelCase")]
    Attribute,
    /// # Save to File
    /// Save response to a file in storage
    #[serde(rename_all = "camelCase")]
    File {
        /// # Output Path
        /// File path to save response to. Supports expressions.
        path: Expr,
        /// # Store Path in Attribute
        /// Whether to store the file path in an attribute (default: true)
        #[serde(skip_serializing_if = "Option::is_none")]
        store_path_in_attribute: Option<bool>,
        /// # Path Attribute Name
        /// Name of attribute to store file path (default: "_response_file_path")
        #[serde(skip_serializing_if = "Option::is_none")]
        path_attribute: Option<String>,
    },
}

/// Response encoding options
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ResponseEncoding {
    /// Text encoding (UTF-8 string)
    Text,
    /// Base64 encoding (for binary data)
    Base64,
    /// Raw bytes (binary)
    Binary,
}

/// Retry configuration
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetryConfig {
    /// # Maximum Attempts
    /// Maximum number of retry attempts (default: 3)
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,

    /// # Initial Delay (milliseconds)
    /// Initial delay before first retry (default: 100ms)
    #[serde(default = "default_initial_delay")]
    pub initial_delay_ms: u64,

    /// # Backoff Multiplier
    /// Multiplier for exponential backoff (default: 2.0)
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,

    /// # Maximum Delay (milliseconds)
    /// Maximum delay between retries (default: 10000ms = 10s)
    #[serde(default = "default_max_delay")]
    pub max_delay_ms: u64,

    /// # Retry On Status Codes
    /// HTTP status codes to retry (default: 5xx)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_on_status: Option<Vec<u16>>,

    /// # Honor Retry-After Header
    /// Respect Retry-After header from server (default: true)
    #[serde(default = "default_honor_retry_after")]
    pub honor_retry_after: bool,
}

fn default_max_attempts() -> u32 {
    3
}

fn default_initial_delay() -> u64 {
    100
}

fn default_backoff_multiplier() -> f64 {
    2.0
}

fn default_max_delay() -> u64 {
    10000
}

fn default_honor_retry_after() -> bool {
    true
}

/// Rate limiting configuration
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitConfig {
    /// # Requests Per Interval
    /// Maximum number of requests allowed per interval
    pub requests: u32,

    /// # Interval (milliseconds)
    /// Time window for rate limit (default: 1000ms = 1 second)
    #[serde(default = "default_rate_interval")]
    pub interval_ms: u64,

    /// # Timing Strategy
    /// How to distribute requests within the interval
    #[serde(default = "default_timing_strategy")]
    pub timing: TimingStrategy,
}

fn default_rate_interval() -> u64 {
    1000
}

fn default_timing_strategy() -> TimingStrategy {
    TimingStrategy::Burst
}

/// Request timing strategy
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum TimingStrategy {
    /// # Burst
    /// Send all requests as fast as possible until limit reached
    Burst,
    /// # Distributed
    /// Distribute requests evenly across the interval
    Distributed,
}

/// Observability configuration
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ObservabilityConfig {
    /// # Track Request Duration
    /// Store request duration in milliseconds (default: true)
    #[serde(default = "default_track_duration")]
    pub track_duration: bool,

    /// # Duration Attribute Name
    /// Name of attribute to store request duration (default: "_request_duration_ms")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_attribute: Option<String>,

    /// # Track Final URL
    /// Store final URL after redirects (default: false)
    #[serde(default = "default_track_final_url")]
    pub track_final_url: bool,

    /// # Final URL Attribute Name
    /// Name of attribute to store final URL (default: "_final_url")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_url_attribute: Option<String>,

    /// # Track Retry Count
    /// Store number of retries performed (default: true when retry enabled)
    #[serde(default = "default_track_retry_count")]
    pub track_retry_count: bool,

    /// # Retry Count Attribute Name
    /// Name of attribute to store retry count (default: "_retry_count")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_count_attribute: Option<String>,

    /// # Track Bytes Transferred
    /// Store response body size in bytes (default: false)
    #[serde(default = "default_track_bytes")]
    pub track_bytes: bool,

    /// # Bytes Attribute Name
    /// Name of attribute to store bytes transferred (default: "_bytes_transferred")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes_attribute: Option<String>,
}

fn default_track_duration() -> bool {
    true
}

fn default_track_final_url() -> bool {
    false
}

fn default_track_retry_count() -> bool {
    true
}

fn default_track_bytes() -> bool {
    false
}

/// Request body types
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RequestBody {
    /// # Text Body
    /// Plain text or JSON body (supports expressions)
    #[serde(rename_all = "camelCase")]
    Text {
        /// # Content
        /// Text content. Supports expressions.
        content: Expr,
        /// # Content-Type
        /// MIME type (e.g., "application/json", "text/plain")
        #[serde(skip_serializing_if = "Option::is_none")]
        content_type: Option<String>,
    },
    /// # Binary Body
    /// Binary data from base64-encoded string or file
    #[serde(rename_all = "camelCase")]
    Binary {
        /// # Source
        /// Binary data source
        source: BinarySource,
        /// # Content-Type
        /// MIME type (e.g., "application/octet-stream")
        #[serde(skip_serializing_if = "Option::is_none")]
        content_type: Option<String>,
    },
    /// # Form URL-Encoded
    /// application/x-www-form-urlencoded
    #[serde(rename_all = "camelCase")]
    FormUrlEncoded {
        /// # Fields
        /// Form fields (name/value pairs)
        fields: Vec<FormField>,
    },
    /// # Multipart Form Data
    /// multipart/form-data with mixed text and file parts
    #[serde(rename_all = "camelCase")]
    Multipart {
        /// # Parts
        /// Multipart form parts
        parts: Vec<MultipartPart>,
    },
}

/// Binary data source
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum BinarySource {
    /// # Base64 Encoded String
    /// Binary data encoded as base64 string
    #[serde(rename_all = "camelCase")]
    Base64 {
        /// # Data
        /// Base64-encoded binary data. Supports expressions.
        data: Expr,
    },
    /// # File Path
    /// Load binary data from a file
    #[serde(rename_all = "camelCase")]
    File {
        /// # Path
        /// File path (supports storage URLs like "ram://", "s3://"). Supports expressions.
        path: Expr,
    },
}

/// Form field for URL-encoded forms
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FormField {
    /// # Field Name
    /// Name of the form field
    pub name: String,
    /// # Field Value
    /// Value of the form field. Supports expressions.
    pub value: Expr,
}

/// Multipart form part
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MultipartPart {
    /// # Text Part
    /// Text field in multipart form
    #[serde(rename_all = "camelCase")]
    Text {
        /// # Field Name
        /// Name of the form field
        name: String,
        /// # Value
        /// Text value. Supports expressions.
        value: Expr,
    },
    /// # File Part
    /// File upload in multipart form
    #[serde(rename_all = "camelCase")]
    File {
        /// # Field Name
        /// Name of the form field
        name: String,
        /// # File Source
        /// Source of the file data
        source: BinarySource,
        /// # File Name
        /// Name of the file to send (optional)
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
        /// # Content-Type
        /// MIME type of the file (optional)
        #[serde(skip_serializing_if = "Option::is_none")]
        content_type: Option<String>,
    },
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
            request_body: Some(RequestBody::Text {
                content: Expr::new(r#"`{"data": "${value}"}`"#),
                content_type: Some("application/json".to_string()),
            }),
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
            response_handling: None,
            max_response_size: None,
            response_encoding: None,
            auto_detect_encoding: None,
            retry: None,
            rate_limit: None,
            observability: None,
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

    #[test]
    fn test_webdav_methods() {
        // Test all WebDAV methods convert correctly
        assert!(matches!(Method::from(HttpMethod::Copy), _));
        assert!(matches!(Method::from(HttpMethod::Lock), _));
        assert!(matches!(Method::from(HttpMethod::Mkcol), _));
        assert!(matches!(Method::from(HttpMethod::Move), _));
        assert!(matches!(Method::from(HttpMethod::Propfind), _));
        assert!(matches!(Method::from(HttpMethod::Proppatch), _));
        assert!(matches!(Method::from(HttpMethod::Unlock), _));
    }

    #[test]
    fn test_text_body_serialization() {
        let body = RequestBody::Text {
            content: Expr::new(r#"{"test": "value"}"#),
            content_type: Some("application/json".to_string()),
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("text"));
        assert!(json.contains("application/json"));
    }

    #[test]
    fn test_binary_body_serialization() {
        let body = RequestBody::Binary {
            source: BinarySource::Base64 {
                data: Expr::new("SGVsbG8="),
            },
            content_type: Some("application/octet-stream".to_string()),
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("binary"));
        assert!(json.contains("base64"));
    }

    #[test]
    fn test_form_body_serialization() {
        let body = RequestBody::FormUrlEncoded {
            fields: vec![FormField {
                name: "field1".to_string(),
                value: Expr::new("value1"),
            }],
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("formUrlEncoded"));
        assert!(json.contains("field1"));
    }

    #[test]
    fn test_multipart_body_serialization() {
        let body = RequestBody::Multipart {
            parts: vec![MultipartPart::Text {
                name: "field1".to_string(),
                value: Expr::new("value1"),
            }],
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("multipart"));
        assert!(json.contains("field1"));
    }

    #[test]
    fn test_response_handling_attribute() {
        let handling = ResponseHandling::Attribute;
        let json = serde_json::to_string(&handling).unwrap();
        assert!(json.contains("attribute"));
    }

    #[test]
    fn test_response_handling_file() {
        let handling = ResponseHandling::File {
            path: Expr::new("ram:///output/response.json"),
            store_path_in_attribute: Some(true),
            path_attribute: Some("_file_path".to_string()),
        };
        let json = serde_json::to_string(&handling).unwrap();
        assert!(json.contains("file"));
        assert!(json.contains("ram:///output/response.json"));
    }

    #[test]
    fn test_response_encoding() {
        let encoding = ResponseEncoding::Base64;
        let json = serde_json::to_string(&encoding).unwrap();
        assert!(json.contains("base64"));
    }

    #[test]
    fn test_full_param_with_response_handling() {
        let params = HttpCallerParam {
            url: Expr::new("https://example.com"),
            method: HttpMethod::Get,
            custom_headers: None,
            query_parameters: None,
            request_body: None,
            content_type: None,
            response_body_attribute: "_response".to_string(),
            status_code_attribute: "_status".to_string(),
            headers_attribute: "_headers".to_string(),
            error_attribute: "_error".to_string(),
            connection_timeout: None,
            transfer_timeout: None,
            authentication: None,
            user_agent: None,
            verify_ssl: None,
            follow_redirects: None,
            max_redirects: None,
            response_handling: Some(ResponseHandling::File {
                path: Expr::new("ram:///output.dat"),
                store_path_in_attribute: Some(true),
                path_attribute: None,
            }),
            max_response_size: Some(1048576), // 1MB
            response_encoding: Some(ResponseEncoding::Base64),
            auto_detect_encoding: Some(false),
            retry: None,
            rate_limit: None,
            observability: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("https://example.com"));
        assert!(json.contains("1048576"));
    }

    #[test]
    fn test_retry_config() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay_ms: 200,
            backoff_multiplier: 3.0,
            max_delay_ms: 5000,
            retry_on_status: Some(vec![429, 503]),
            honor_retry_after: false,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("maxAttempts"));
        assert!(json.contains("5"));
    }

    #[test]
    fn test_rate_limit_config() {
        let config = RateLimitConfig {
            requests: 10,
            interval_ms: 1000,
            timing: TimingStrategy::Distributed,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("requests"));
        assert!(json.contains("10"));
        assert!(json.contains("distributed"));
    }

    #[test]
    fn test_observability_config() {
        let config = ObservabilityConfig {
            track_duration: true,
            duration_attribute: Some("_time".to_string()),
            track_final_url: true,
            final_url_attribute: Some("_url".to_string()),
            track_retry_count: true,
            retry_count_attribute: None,
            track_bytes: true,
            bytes_attribute: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("trackDuration"));
        assert!(json.contains("_time"));
    }
}
