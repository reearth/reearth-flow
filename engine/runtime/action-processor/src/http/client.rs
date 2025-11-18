use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use reqwest::Method;

use super::errors::{HttpProcessorError, Result};

pub(crate) trait HttpClient: Send + Sync {
    /// Send an HTTP request
    fn send_request(
        &self,
        method: Method,
        url: &str,
        headers: HeaderMap,
        query_params: Vec<(String, String)>,
        body: Option<String>,
    ) -> Result<HttpResponse>;
}

/// HTTP response data
#[derive(Debug, Clone)]
pub(crate) struct HttpResponse {
    pub status_code: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body: String,
}

#[derive(Clone)]
pub(crate) struct ReqwestHttpClient {
    client: Client,
}

impl ReqwestHttpClient {
    pub fn new(connection_timeout: u64, transfer_timeout: u64) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(transfer_timeout))
            .connect_timeout(Duration::from_secs(connection_timeout))
            .redirect(reqwest::redirect::Policy::limited(10))
            .user_agent("reearth-flow-http-caller/1.0")
            .build()
            .map_err(|e| {
                HttpProcessorError::CallerFactory(format!("Failed to create HTTP client: {e}"))
            })?;

        Ok(Self { client })
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn with_client(client: Client) -> Self {
        Self { client }
    }
}

impl HttpClient for ReqwestHttpClient {
    fn send_request(
        &self,
        method: Method,
        url: &str,
        headers: HeaderMap,
        query_params: Vec<(String, String)>,
        body: Option<String>,
    ) -> Result<HttpResponse> {
        let mut request_builder = self.client.request(method, url);
        request_builder = request_builder.headers(headers);

        if !query_params.is_empty() {
            request_builder = request_builder.query(&query_params);
        }

        if let Some(body_content) = body {
            request_builder = request_builder.body(body_content);
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

        let body = response.text().map_err(|e| {
            HttpProcessorError::Response(format!("Failed to read response body: {e}"))
        })?;

        Ok(HttpResponse {
            status_code,
            headers: response_headers,
            body,
        })
    }
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
        _body: Option<String>,
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
            body: r#"{"status": "ok"}"#.to_string(),
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
        assert_eq!(resp.body, r#"{"status": "ok"}"#);
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
}
