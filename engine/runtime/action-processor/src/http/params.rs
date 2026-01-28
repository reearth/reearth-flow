use reearth_flow_types::Expr;
use reqwest::Method;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// # HTTP Caller Parameters
/// Configure HTTP/HTTPS requests to enrich features with response data
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HttpCallerParam {
    /// # URL
    /// The target URL for the HTTP request (supports expressions)
    pub url: Expr,

    /// # HTTP Method
    /// The HTTP method to use for the request
    #[serde(default = "default_method")]
    pub method: HttpMethod,

    /// # Authentication
    /// Authentication method and credentials for the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<Authentication>,

    /// # Custom Headers
    /// Additional HTTP headers to include in the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_headers: Option<Vec<HeaderParam>>,

    /// # Query Parameters
    /// URL query parameters to append to the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_parameters: Option<Vec<QueryParam>>,

    /// # Request Body
    /// The body content to send with the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBody>,

    /// # Content Type
    /// Override the Content-Type header for the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    /// # Timeouts
    /// Connection and transfer timeout settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeouts: Option<TimeoutConfig>,

    /// # HTTP Options
    /// HTTP client behavior settings (SSL, redirects, user agent)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_options: Option<HttpOptions>,

    /// # Response Configuration
    /// Configure how response data is stored and processed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<ResponseConfig>,

    /// # Retry Configuration
    /// Settings for automatic retry on failures
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,

    /// # Rate Limiting
    /// Rate limiting configuration to control request frequency
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimitConfig>,

    /// # Observability
    /// Track additional metrics and diagnostics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observability: Option<ObservabilityConfig>,
}

fn default_method() -> HttpMethod {
    HttpMethod::Get
}

/// # Timeout Configuration
/// Configure connection and transfer timeouts for HTTP requests
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TimeoutConfig {
    /// # Connection Timeout
    /// Maximum time in seconds to establish a connection (default: 60)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_timeout: Option<u64>,

    /// # Transfer Timeout
    /// Maximum time in seconds to complete the entire request (default: 90)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transfer_timeout: Option<u64>,
}

/// # HTTP Options
/// Configure HTTP client behavior
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HttpOptions {
    /// # User Agent
    /// Custom User-Agent header value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    /// # Verify SSL
    /// Whether to verify SSL/TLS certificates (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_ssl: Option<bool>,

    /// # Follow Redirects
    /// Whether to automatically follow HTTP redirects (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_redirects: Option<bool>,

    /// # Max Redirects
    /// Maximum number of redirects to follow (default: 10)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_redirects: Option<u8>,
}

/// # Response Configuration
/// Configure how HTTP response data is stored and processed
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResponseConfig {
    /// # Response Body Attribute
    /// Feature attribute name to store the response body (default: "_response_body")
    #[serde(default = "default_response_body_attr")]
    pub response_body_attribute: String,

    /// # Status Code Attribute
    /// Feature attribute name to store the HTTP status code (default: "_http_status_code")
    #[serde(default = "default_status_code_attr")]
    pub status_code_attribute: String,

    /// # Headers Attribute
    /// Feature attribute name to store the response headers (default: "_headers")
    #[serde(default = "default_headers_attr")]
    pub headers_attribute: String,

    /// # Error Attribute
    /// Feature attribute name to store any error messages (default: "_http_error")
    #[serde(default = "default_error_attr")]
    pub error_attribute: String,

    /// # Response Handling
    /// How to handle the response data (attribute or file)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_handling: Option<ResponseHandling>,

    /// # Max Response Size
    /// Maximum response body size in bytes (unlimited if not set)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_response_size: Option<u64>,

    /// # Response Encoding
    /// How to encode the response body (text, base64, or binary)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_encoding: Option<ResponseEncoding>,

    /// # Auto Detect Encoding
    /// Automatically detect character encoding from response headers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_detect_encoding: Option<bool>,
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

/// # HTTP Method
/// The HTTP request method to use
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    /// # GET
    /// Retrieve data from the server
    Get,
    /// # POST
    /// Submit data to the server
    Post,
    /// # PUT
    /// Update or create a resource
    Put,
    /// # DELETE
    /// Delete a resource
    Delete,
    /// # PATCH
    /// Partially update a resource
    Patch,
    /// # HEAD
    /// Retrieve headers only (no body)
    Head,
    /// # OPTIONS
    /// Query supported methods
    Options,
    /// # COPY
    /// WebDAV: Copy a resource
    Copy,
    /// # LOCK
    /// WebDAV: Lock a resource
    Lock,
    /// # MKCOL
    /// WebDAV: Create a collection
    Mkcol,
    /// # MOVE
    /// WebDAV: Move a resource
    Move,
    /// # PROPFIND
    /// WebDAV: Retrieve properties
    Propfind,
    /// # PROPPATCH
    /// WebDAV: Update properties
    Proppatch,
    /// # UNLOCK
    /// WebDAV: Unlock a resource
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

/// # HTTP Header
/// A custom HTTP header to include in the request
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HeaderParam {
    /// # Header Name
    /// The name of the HTTP header
    pub name: String,
    /// # Header Value
    /// The value of the header (supports expressions)
    pub value: Expr,
}

/// # Query Parameter
/// A URL query parameter to append to the request
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct QueryParam {
    /// # Parameter Name
    /// The name of the query parameter
    pub name: String,
    /// # Parameter Value
    /// The value of the parameter (supports expressions)
    pub value: Expr,
}

/// # Authentication
/// Authentication method and credentials for HTTP requests
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Authentication {
    /// # Basic Authentication
    /// HTTP Basic authentication with username and password
    #[serde(rename_all = "camelCase")]
    Basic {
        /// # Username
        /// The username for basic authentication
        username: Expr,
        /// # Password
        /// The password for basic authentication
        password: Expr,
    },
    /// # Bearer Token
    /// Bearer token authentication (OAuth 2.0)
    #[serde(rename_all = "camelCase")]
    Bearer {
        /// # Token
        /// The bearer token value
        token: Expr,
    },
    /// # API Key
    /// API key authentication in header or query parameter
    #[serde(rename_all = "camelCase")]
    ApiKey {
        /// # Key Name
        /// The name of the API key parameter
        key_name: String,
        /// # Key Value
        /// The API key value
        key_value: Expr,
        /// # Location
        /// Where to include the API key (header or query parameter)
        #[serde(default = "default_api_key_location")]
        location: ApiKeyLocation,
    },
}

/// # API Key Location
/// Where to include the API key in the request
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ApiKeyLocation {
    /// # Header
    /// Include API key in HTTP header
    Header,
    /// # Query Parameter
    /// Include API key in URL query string
    Query,
}

fn default_api_key_location() -> ApiKeyLocation {
    ApiKeyLocation::Header
}

/// # Response Handling
/// How to handle the HTTP response data
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ResponseHandling {
    /// # Store in Attribute
    /// Store response body in a feature attribute
    #[serde(rename_all = "camelCase")]
    Attribute,
    /// # Save to File
    /// Save response body to a file
    #[serde(rename_all = "camelCase")]
    File {
        /// # File Path
        /// Path where the response should be saved
        path: Expr,
        /// # Store Path in Attribute
        /// Whether to store the file path in a feature attribute
        #[serde(skip_serializing_if = "Option::is_none")]
        store_path_in_attribute: Option<bool>,
        /// # Path Attribute Name
        /// Attribute name for storing the file path
        #[serde(skip_serializing_if = "Option::is_none")]
        path_attribute: Option<String>,
    },
}

/// # Response Encoding
/// How to encode the response body data
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ResponseEncoding {
    /// # Text
    /// Decode response as UTF-8 text
    Text,
    /// # Base64
    /// Encode response as base64 string
    Base64,
    /// # Binary
    /// Store response as raw binary data
    Binary,
}

/// # Retry Configuration
/// Configure automatic retry behavior for failed requests
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetryConfig {
    /// # Max Attempts
    /// Maximum number of retry attempts (default: 3)
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,

    /// # Initial Delay
    /// Initial delay in milliseconds before first retry (default: 100ms)
    #[serde(default = "default_initial_delay")]
    pub initial_delay_ms: u64,

    /// # Backoff Multiplier
    /// Multiplier for exponential backoff between retries (default: 2.0)
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,

    /// # Max Delay
    /// Maximum delay in milliseconds between retries (default: 10000ms)
    #[serde(default = "default_max_delay")]
    pub max_delay_ms: u64,

    /// # Retry on Status Codes
    /// List of HTTP status codes that should trigger a retry (e.g., [429, 503])
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_on_status: Option<Vec<u16>>,

    /// # Honor Retry-After Header
    /// Whether to respect the Retry-After header from server responses (default: true)
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

/// # Rate Limit Configuration
/// Control the rate of HTTP requests to avoid overwhelming the server
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitConfig {
    /// # Requests
    /// Maximum number of requests allowed within the interval
    pub requests: u32,

    /// # Interval
    /// Time interval in milliseconds for the rate limit (default: 1000ms)
    #[serde(default = "default_rate_interval")]
    pub interval_ms: u64,

    /// # Timing Strategy
    /// How to distribute requests within the interval (default: Burst)
    #[serde(default = "default_timing_strategy")]
    pub timing: TimingStrategy,
}

fn default_rate_interval() -> u64 {
    1000
}

fn default_timing_strategy() -> TimingStrategy {
    TimingStrategy::Burst
}

/// # Timing Strategy
/// How to distribute requests within the rate limit interval
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum TimingStrategy {
    /// # Burst
    /// Allow all requests immediately, then pause until next interval
    Burst,
    /// # Distributed
    /// Evenly distribute requests throughout the interval
    Distributed,
}

/// # Observability Configuration
/// Track additional metrics and diagnostics about HTTP requests
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ObservabilityConfig {
    /// # Track Duration
    /// Whether to track the total request duration (default: true)
    #[serde(default = "default_track_duration")]
    pub track_duration: bool,

    /// # Duration Attribute
    /// Feature attribute name to store request duration in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_attribute: Option<String>,

    /// # Track Final URL
    /// Whether to track the final URL after redirects (default: false)
    #[serde(default = "default_track_final_url")]
    pub track_final_url: bool,

    /// # Final URL Attribute
    /// Feature attribute name to store the final URL after redirects
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_url_attribute: Option<String>,

    /// # Track Retry Count
    /// Whether to track the number of retry attempts (default: true)
    #[serde(default = "default_track_retry_count")]
    pub track_retry_count: bool,

    /// # Retry Count Attribute
    /// Feature attribute name to store the number of retry attempts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_count_attribute: Option<String>,

    /// # Track Bytes
    /// Whether to track the response body size in bytes (default: false)
    #[serde(default = "default_track_bytes")]
    pub track_bytes: bool,

    /// # Bytes Attribute
    /// Feature attribute name to store the response body size
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

/// # Request Body
/// The body content to send with the HTTP request
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RequestBody {
    /// # Text Body
    /// Send text or JSON content
    #[serde(rename_all = "camelCase")]
    Text {
        /// # Content
        /// The text content to send (supports expressions)
        content: Expr,
        /// # Content Type
        /// Override Content-Type header (e.g., application/json, text/plain)
        #[serde(skip_serializing_if = "Option::is_none")]
        content_type: Option<String>,
    },
    /// # Binary Body
    /// Send binary data from base64 or file
    #[serde(rename_all = "camelCase")]
    Binary {
        /// # Binary Source
        /// Source of the binary data (base64 string or file path)
        source: BinarySource,
        /// # Content Type
        /// Content-Type header (e.g., application/octet-stream, image/png)
        #[serde(skip_serializing_if = "Option::is_none")]
        content_type: Option<String>,
    },
    /// # Form URL Encoded
    /// Send application/x-www-form-urlencoded data
    #[serde(rename_all = "camelCase")]
    FormUrlEncoded {
        /// # Form Fields
        /// List of form field name-value pairs
        fields: Vec<FormField>,
    },
    /// # Multipart Form Data
    /// Send multipart/form-data (for file uploads)
    #[serde(rename_all = "camelCase")]
    Multipart {
        /// # Parts
        /// List of multipart form parts (text fields or file uploads)
        parts: Vec<MultipartPart>,
    },
}

/// # Binary Source
/// Source of binary data for request body
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum BinarySource {
    /// # Base64 Encoded
    /// Binary data encoded as base64 string
    #[serde(rename_all = "camelCase")]
    Base64 {
        /// # Data
        /// Base64-encoded binary data (supports expressions)
        data: Expr,
    },
    /// # From File
    /// Read binary data from a file
    #[serde(rename_all = "camelCase")]
    File {
        /// # File Path
        /// Path to the file to read (supports expressions)
        path: Expr,
    },
}

/// # Form Field
/// A name-value pair for URL-encoded form data
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FormField {
    /// # Field Name
    /// The name of the form field
    pub name: String,
    /// # Field Value
    /// The value of the form field (supports expressions)
    pub value: Expr,
}

/// # Multipart Part
/// A part in a multipart/form-data request
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MultipartPart {
    /// # Text Field
    /// A text field in the multipart form
    #[serde(rename_all = "camelCase")]
    Text {
        /// # Field Name
        /// The name of the form field
        name: String,
        /// # Field Value
        /// The value of the form field (supports expressions)
        value: Expr,
    },
    /// # File Upload
    /// A file upload in the multipart form
    #[serde(rename_all = "camelCase")]
    File {
        /// # Field Name
        /// The name of the file upload field
        name: String,
        /// # File Source
        /// Source of the file data (base64 or file path)
        source: BinarySource,
        /// # Filename
        /// The filename to send in the Content-Disposition header
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
        /// # Content Type
        /// MIME type of the file
        #[serde(skip_serializing_if = "Option::is_none")]
        content_type: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webdav_method_conversion() {
        assert_eq!(
            Method::from(HttpMethod::Propfind),
            Method::from_bytes(b"PROPFIND").unwrap()
        );
    }

    #[test]
    fn test_api_key_default_location() {
        assert!(matches!(default_api_key_location(), ApiKeyLocation::Header));
    }
}
