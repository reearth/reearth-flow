use std::io::Read;
use std::sync::Arc;
use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use reqwest::Method;

use super::body::BodyContent;
use super::egress::{redirect_policy, EgressGuardedDnsResolver};
use super::errors::{HttpProcessorError, Result};

pub(crate) trait HttpClient: Send + Sync {
    fn send_request(
        &self,
        method: Method,
        url: &str,
        headers: HeaderMap,
        query_params: Vec<(String, String)>,
        body: Option<BodyContent>,
    ) -> Result<HttpResponse>;
}

#[derive(Debug, Clone)]
pub(crate) struct HttpResponse {
    pub status_code: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Clone)]
pub(crate) struct ReqwestHttpClient {
    client: Client,
    max_response_size: Option<u64>,
}

#[derive(Clone)]
pub(crate) struct ClientConfig {
    pub connection_timeout: u64,
    pub transfer_timeout: u64,
    pub user_agent: Option<String>,
    pub verify_ssl: bool,
    pub follow_redirects: bool,
    pub max_redirects: u8,
    pub max_response_size: Option<u64>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            connection_timeout: 60,
            transfer_timeout: 90,
            user_agent: None,
            verify_ssl: true,
            follow_redirects: true,
            max_redirects: 10,
            max_response_size: None,
        }
    }
}

impl ReqwestHttpClient {
    pub fn with_config(config: ClientConfig) -> Result<Self> {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(config.transfer_timeout))
            .connect_timeout(Duration::from_secs(config.connection_timeout))
            .danger_accept_invalid_certs(!config.verify_ssl)
            .dns_resolver(Arc::new(EgressGuardedDnsResolver));

        if config.follow_redirects {
            builder = builder.redirect(redirect_policy(config.max_redirects as usize));
        } else {
            builder = builder.redirect(reqwest::redirect::Policy::none());
        }

        let user_agent = config
            .user_agent
            .unwrap_or_else(|| "reearth-flow-http-caller/1.0".to_string());
        builder = builder.user_agent(user_agent);

        let client = builder.build().map_err(|e| {
            HttpProcessorError::CallerFactory(format!("Failed to create HTTP client: {e}"))
        })?;

        Ok(Self {
            client,
            max_response_size: config.max_response_size,
        })
    }
}

impl HttpClient for ReqwestHttpClient {
    fn send_request(
        &self,
        method: Method,
        url: &str,
        headers: HeaderMap,
        query_params: Vec<(String, String)>,
        body: Option<BodyContent>,
    ) -> Result<HttpResponse> {
        let mut request_builder = self.client.request(method, url);
        request_builder = request_builder.headers(headers);

        if !query_params.is_empty() {
            request_builder = request_builder.query(&query_params);
        }

        // Handle different body types
        if let Some(body_content) = body {
            request_builder = match body_content {
                BodyContent::Text(text) => request_builder.body(text),
                BodyContent::Binary(data) => request_builder.body(data),
                BodyContent::Form(fields) => request_builder.form(&fields),
                BodyContent::Multipart(form) => request_builder.multipart(form),
            };
        }

        let response = request_builder
            .send()
            .map_err(|e| HttpProcessorError::Request(format!("HTTP request failed: {e}")))?;

        let status_code = response.status().as_u16();

        let response_headers: std::collections::HashMap<String, String> = response
            .headers()
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.to_string(), v.to_string())))
            .collect();

        let content_length = response.content_length();
        let body = read_body_capped(response, content_length, self.max_response_size)?;

        Ok(HttpResponse {
            status_code,
            headers: response_headers,
            body,
        })
    }
}

/// Read the response body, aborting the download as soon as `max_size` is
/// exceeded rather than buffering the whole body first.
fn read_body_capped(
    mut reader: impl Read,
    content_length: Option<u64>,
    max_size: Option<u64>,
) -> Result<Vec<u8>> {
    let Some(max_size) = max_size else {
        let mut body = Vec::new();
        reader.read_to_end(&mut body).map_err(|e| {
            HttpProcessorError::Response(format!("Failed to read response body: {e}"))
        })?;
        return Ok(body);
    };

    if let Some(len) = content_length {
        if len > max_size {
            return Err(size_exceeded(len, max_size));
        }
    }

    let mut body = Vec::new();
    reader
        .take(max_size + 1)
        .read_to_end(&mut body)
        .map_err(|e| HttpProcessorError::Response(format!("Failed to read response body: {e}")))?;

    if body.len() as u64 > max_size {
        return Err(size_exceeded(body.len() as u64, max_size));
    }
    Ok(body)
}

fn size_exceeded(size: u64, max_size: u64) -> HttpProcessorError {
    HttpProcessorError::Response(format!(
        "Response body size ({size} bytes) exceeds maximum allowed size ({max_size} bytes)"
    ))
}

/// Mock HTTP client for testing
#[cfg(test)]
#[derive(Clone)]
pub(crate) struct MockHttpClient {
    pub responses: std::collections::HashMap<String, Result<HttpResponse>>,
}

#[cfg(test)]
impl MockHttpClient {
    pub fn new() -> Self {
        Self {
            responses: std::collections::HashMap::new(),
        }
    }

    pub fn with_response(mut self, url: &str, response: Result<HttpResponse>) -> Self {
        self.responses.insert(url.to_string(), response);
        self
    }
}

#[cfg(test)]
impl HttpClient for MockHttpClient {
    fn send_request(
        &self,
        _method: Method,
        url: &str,
        _headers: HeaderMap,
        _query_params: Vec<(String, String)>,
        _body: Option<BodyContent>,
    ) -> Result<HttpResponse> {
        self.responses.get(url).cloned().unwrap_or_else(|| {
            Err(HttpProcessorError::Request(format!(
                "No mock response for URL: {url}"
            )))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_http_client() {
        let response = HttpResponse {
            status_code: 200,
            headers: std::collections::HashMap::from([(
                "content-type".to_string(),
                "application/json".to_string(),
            )]),
            body: r#"{"status": "ok"}"#.as_bytes().to_vec(),
        };

        let mock =
            MockHttpClient::new().with_response("https://example.com/test", Ok(response.clone()));

        let result = mock.send_request(
            Method::GET,
            "https://example.com/test",
            HeaderMap::new(),
            vec![],
            None,
        );

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.body, r#"{"status": "ok"}"#.as_bytes());
    }

    #[test]
    fn test_mock_http_client_error() {
        let mock = MockHttpClient::new();

        let result = mock.send_request(
            Method::GET,
            "https://notfound.com",
            HeaderMap::new(),
            vec![],
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_read_body_uncapped() {
        let data = vec![b'a'; 1000];
        let body = read_body_capped(data.as_slice(), None, None).unwrap();
        assert_eq!(body.len(), 1000);
    }

    #[test]
    fn test_read_body_rejected_by_content_length() {
        let data = vec![b'a'; 1000];
        let err = read_body_capped(data.as_slice(), Some(1000), Some(500)).unwrap_err();
        assert!(err.to_string().contains("exceeds maximum"));
    }

    #[test]
    fn test_read_body_stops_at_cap_without_content_length() {
        // No Content-Length (e.g. chunked encoding): the read itself must stop
        // shortly past the cap instead of buffering the full body.
        let data = vec![b'a'; 10_000];
        let err = read_body_capped(data.as_slice(), None, Some(500)).unwrap_err();
        assert!(err.to_string().contains("exceeds maximum"));
    }

    #[test]
    fn test_read_body_within_cap() {
        let data = vec![b'a'; 400];
        let body = read_body_capped(data.as_slice(), Some(400), Some(500)).unwrap();
        assert_eq!(body.len(), 400);
    }
}
