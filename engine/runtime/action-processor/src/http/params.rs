use reearth_flow_types::Code;
use reqwest::Method;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// # HTTP Caller Parameters
/// Configure the HTTP request made for each feature and how the response is stored
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HttpCallerParam {
    /// # URL
    /// The URL to request, evaluated for each feature (supports expressions). Only http and https URLs are allowed, and requests to private or internal network addresses are blocked.
    pub url: Code,

    /// # HTTP Method
    /// The HTTP method to use for the request (default: GET)
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

    /// # Response Configuration
    /// Configure how the response is stored on the feature
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

    /// # Timeouts
    /// Connection and transfer timeout settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeouts: Option<TimeoutConfig>,

    /// # HTTP Options
    /// HTTP client behavior settings (SSL verification, redirects, user agent)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_options: Option<HttpOptions>,
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
    /// Custom User-Agent header value sent with each request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    /// # Verify SSL
    /// Whether to verify SSL/TLS certificates; disable only for servers with self-signed certificates (default: true)
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
/// Configure how the response is stored. The status code, response headers, and any error message are always stored in the `_http_status_code`, `_headers`, and `_http_error` attributes.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResponseConfig {
    /// # Response Body Attribute
    /// Feature attribute name to store the response body (default: `_response_body`)
    #[serde(default = "default_response_body_attr")]
    pub response_body_attribute: String,

    /// # Response Handling
    /// Whether to store the response body in a feature attribute or save it to a file (default: attribute)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_handling: Option<ResponseHandling>,

    /// # Response Encoding
    /// How to store the response body: as UTF-8 text or as a base64-encoded string. When omitted, the encoding is chosen from the response's Content-Type header.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_encoding: Option<ResponseEncoding>,

    /// # Auto Detect Encoding
    /// Choose text or base64 storage from the response's Content-Type header when Response Encoding is not set (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_detect_encoding: Option<bool>,

    /// # Max Response Size
    /// Maximum response body size in bytes; the download is stopped and the feature rejected when a response exceeds it (unlimited if not set)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_response_size: Option<u64>,
}

pub(crate) fn default_response_body_attr() -> String {
    "_response_body".to_string()
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
    pub value: Code,
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
    pub value: Code,
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
        username: Code,
        /// # Password
        /// The password for basic authentication
        password: Code,
    },
    /// # Bearer Token
    /// Bearer token authentication (OAuth 2.0)
    #[serde(rename_all = "camelCase")]
    Bearer {
        /// # Token
        /// The bearer token value
        token: Code,
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
        key_value: Code,
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
    /// Store the response body in a feature attribute
    #[serde(rename_all = "camelCase")]
    Attribute,
    /// # Save to File
    /// Save the response body to a file under the job's output directory, recording its location in the `_response_file_path` attribute
    #[serde(rename_all = "camelCase")]
    File {
        /// # File Path
        /// Relative path under the job's output directory where the response is saved (supports expressions)
        path: Code,
    },
}

/// # Response Encoding
/// How to store the response body
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ResponseEncoding {
    /// # Text
    /// Store the response body as UTF-8 text
    Text,
    /// # Base64
    /// Store the response body as a base64-encoded string (for binary data)
    Base64,
}

/// # Retry Configuration
/// Configure automatic retry behavior for failed requests
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetryConfig {
    /// # Max Attempts
    /// Maximum total number of attempts including the initial request; 1 disables retries (default: 3)
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,

    /// # Initial Delay
    /// Initial delay in milliseconds before the first retry (default: 100)
    #[serde(default = "default_initial_delay")]
    pub initial_delay_ms: u64,

    /// # Backoff Multiplier
    /// Multiplier for exponential backoff between retries (default: 2.0)
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,

    /// # Max Delay
    /// Maximum delay in milliseconds between retries, also capping delays requested by the Retry-After header (default: 10000)
    #[serde(default = "default_max_delay")]
    pub max_delay_ms: u64,

    /// # Retry on Status Codes
    /// HTTP status codes that trigger a retry, such as [429, 503]. When omitted, all 5xx status codes are retried.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_on_status: Option<Vec<u16>>,

    /// # Honor Retry-After Header
    /// Whether to respect the Retry-After response header, in seconds or HTTP-date form, when scheduling a retry (default: true)
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
    /// Time interval in milliseconds for the rate limit (default: 1000)
    #[serde(default = "default_rate_interval")]
    pub interval_ms: u64,

    /// # Timing Strategy
    /// How to distribute requests within the interval (default: burst)
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
        content: Code,
        /// # Content Type
        /// Content-Type header for the body, such as application/json or text/plain
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
        /// Content-Type header for the body (default: application/octet-stream)
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
    /// Send multipart/form-data (for file uploads); cannot be combined with retry
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
        data: Code,
    },
    /// # From File
    /// Read binary data from a file
    #[serde(rename_all = "camelCase")]
    File {
        /// # File Path
        /// Path to the file to read (supports expressions)
        path: Code,
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
    pub value: Code,
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
        value: Code,
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
    fn test_method_conversion() {
        assert_eq!(Method::from(HttpMethod::Get), Method::GET);
        assert_eq!(Method::from(HttpMethod::Patch), Method::PATCH);
    }

    #[test]
    fn test_api_key_default_location() {
        assert!(matches!(default_api_key_location(), ApiKeyLocation::Header));
    }
}
