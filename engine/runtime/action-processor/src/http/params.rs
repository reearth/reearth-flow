use reearth_flow_types::Expr;
use reqwest::Method;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HttpCallerParam {
    pub url: Expr,

    #[serde(default = "default_method")]
    pub method: HttpMethod,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_headers: Option<Vec<HeaderParam>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_parameters: Option<Vec<QueryParam>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBody>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    #[serde(default = "default_response_body_attr")]
    pub response_body_attribute: String,

    #[serde(default = "default_status_code_attr")]
    pub status_code_attribute: String,

    #[serde(default = "default_headers_attr")]
    pub headers_attribute: String,

    #[serde(default = "default_error_attr")]
    pub error_attribute: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_timeout: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub transfer_timeout: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<Authentication>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_ssl: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_redirects: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_redirects: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_handling: Option<ResponseHandling>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_response_size: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_encoding: Option<ResponseEncoding>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_detect_encoding: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimitConfig>,

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

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HeaderParam {
    pub name: String,
    pub value: Expr,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct QueryParam {
    pub name: String,
    pub value: Expr,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Authentication {
    #[serde(rename_all = "camelCase")]
    Basic { username: Expr, password: Expr },
    #[serde(rename_all = "camelCase")]
    Bearer { token: Expr },
    #[serde(rename_all = "camelCase")]
    ApiKey {
        key_name: String,
        key_value: Expr,
        #[serde(default = "default_api_key_location")]
        location: ApiKeyLocation,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ApiKeyLocation {
    Header,
    Query,
}

fn default_api_key_location() -> ApiKeyLocation {
    ApiKeyLocation::Header
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ResponseHandling {
    #[serde(rename_all = "camelCase")]
    Attribute,
    #[serde(rename_all = "camelCase")]
    File {
        path: Expr,
        #[serde(skip_serializing_if = "Option::is_none")]
        store_path_in_attribute: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        path_attribute: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ResponseEncoding {
    Text,
    Base64,
    Binary,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RetryConfig {
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,

    #[serde(default = "default_initial_delay")]
    pub initial_delay_ms: u64,

    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,

    #[serde(default = "default_max_delay")]
    pub max_delay_ms: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_on_status: Option<Vec<u16>>,

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

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitConfig {
    pub requests: u32,

    #[serde(default = "default_rate_interval")]
    pub interval_ms: u64,

    #[serde(default = "default_timing_strategy")]
    pub timing: TimingStrategy,
}

fn default_rate_interval() -> u64 {
    1000
}

fn default_timing_strategy() -> TimingStrategy {
    TimingStrategy::Burst
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum TimingStrategy {
    Burst,
    Distributed,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ObservabilityConfig {
    #[serde(default = "default_track_duration")]
    pub track_duration: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_attribute: Option<String>,

    #[serde(default = "default_track_final_url")]
    pub track_final_url: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_url_attribute: Option<String>,

    #[serde(default = "default_track_retry_count")]
    pub track_retry_count: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_count_attribute: Option<String>,

    #[serde(default = "default_track_bytes")]
    pub track_bytes: bool,

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

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RequestBody {
    #[serde(rename_all = "camelCase")]
    Text {
        content: Expr,
        #[serde(skip_serializing_if = "Option::is_none")]
        content_type: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Binary {
        source: BinarySource,
        #[serde(skip_serializing_if = "Option::is_none")]
        content_type: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    FormUrlEncoded { fields: Vec<FormField> },
    #[serde(rename_all = "camelCase")]
    Multipart { parts: Vec<MultipartPart> },
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum BinarySource {
    #[serde(rename_all = "camelCase")]
    Base64 { data: Expr },
    #[serde(rename_all = "camelCase")]
    File { path: Expr },
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FormField {
    pub name: String,
    pub value: Expr,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MultipartPart {
    #[serde(rename_all = "camelCase")]
    Text { name: String, value: Expr },
    #[serde(rename_all = "camelCase")]
    File {
        name: String,
        source: BinarySource,
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
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
