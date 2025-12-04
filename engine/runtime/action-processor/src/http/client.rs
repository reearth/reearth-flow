//! HTTP client trait and implementations for HTTPCaller

use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::errors::HttpProcessorError;
use super::types::{HttpRequest, HttpResponse};

pub trait HttpClient: Send + Sync {
    /// Execute an HTTP request and return the response
    fn execute(&self, request: &HttpRequest) -> Result<HttpResponse, HttpProcessorError>;
}

/// Configuration for the HTTP client
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub connection_timeout: Duration,
    pub transfer_timeout: Duration,
    pub user_agent: String,
    pub max_redirects: usize,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            connection_timeout: Duration::from_secs(60),
            transfer_timeout: Duration::from_secs(90),
            user_agent: "reearth-flow-http-caller/1.0".to_string(),
            max_redirects: 10,
        }
    }
}

/// Default HTTP client implementation using reqwest
#[derive(Clone)]
pub struct ReqwestHttpClient {
    client: Arc<Mutex<Option<reqwest::blocking::Client>>>,
    config: HttpClientConfig,
}

impl ReqwestHttpClient {
    /// Create a new ReqwestHttpClient with the given configuration
    pub fn new(config: HttpClientConfig) -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            config,
        }
    }

    /// Get or create the underlying reqwest client
    fn get_or_create_client(&self) -> Result<reqwest::blocking::Client, HttpProcessorError> {
        let mut guard = self.client.lock().unwrap();
        if let Some(client) = guard.as_ref() {
            return Ok(client.clone());
        }

        let client = reqwest::blocking::Client::builder()
            .connect_timeout(self.config.connection_timeout)
            .timeout(self.config.transfer_timeout)
            .redirect(reqwest::redirect::Policy::limited(
                self.config.max_redirects,
            ))
            .user_agent(&self.config.user_agent)
            .build()
            .map_err(|e| {
                HttpProcessorError::HttpCaller(format!("Failed to create HTTP client: {e}"))
            })?;

        *guard = Some(client.clone());
        Ok(client)
    }
}

impl HttpClient for ReqwestHttpClient {
    fn execute(&self, request: &HttpRequest) -> Result<HttpResponse, HttpProcessorError> {
        let client = self.get_or_create_client()?;

        // Build the request
        let mut req = client.request(request.method.to_reqwest_method(), &request.url);

        // Add query parameters
        for (name, value) in &request.query_params {
            req = req.query(&[(name, value)]);
        }

        // Add headers
        for (name, value) in &request.headers {
            req = req.header(name, value);
        }

        // Add Content-Type header if specified
        if let Some(ref content_type) = request.content_type {
            req = req.header("Content-Type", content_type);
        }

        // Add request body for methods that support it
        if let Some(ref body) = request.body {
            if request.method.supports_body() {
                req = req.body(body.clone());
            }
        }

        // Execute the request
        let response = req
            .send()
            .map_err(|e| HttpProcessorError::Request(format!("HTTP request failed: {e}")))?;

        // Extract response data
        let status_code = response.status().as_u16();
        let headers: std::collections::HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let body = response.text().map_err(|e| {
            HttpProcessorError::Response(format!("Failed to read response body: {e}"))
        })?;

        Ok(HttpResponse {
            body,
            status_code,
            headers,
        })
    }
}

impl Default for ReqwestHttpClient {
    fn default() -> Self {
        Self::new(HttpClientConfig::default())
    }
}

/// Mock HTTP client for testing purposes
#[cfg(test)]
pub struct MockHttpClient {
    response: Result<HttpResponse, HttpProcessorError>,
    pub requests: Arc<Mutex<Vec<HttpRequest>>>,
}

#[cfg(test)]
impl MockHttpClient {
    /// Create a new mock client that returns the given response
    pub fn new(response: Result<HttpResponse, HttpProcessorError>) -> Self {
        Self {
            response,
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a mock client that returns a successful response
    pub fn with_success(body: String, status_code: u16) -> Self {
        Self::new(Ok(HttpResponse {
            body,
            status_code,
            headers: std::collections::HashMap::new(),
        }))
    }

    /// Create a mock client that returns an error
    pub fn with_error(error: HttpProcessorError) -> Self {
        Self::new(Err(error))
    }

    /// Get the captured requests
    pub fn get_requests(&self) -> Vec<HttpRequest> {
        self.requests.lock().unwrap().clone()
    }
}

#[cfg(test)]
impl HttpClient for MockHttpClient {
    fn execute(&self, request: &HttpRequest) -> Result<HttpResponse, HttpProcessorError> {
        // Capture the request
        self.requests.lock().unwrap().push(request.clone());

        // Return the configured response
        self.response.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::types::HttpMethod;

    #[test]
    fn test_http_client_config_default() {
        let config = HttpClientConfig::default();
        assert_eq!(config.connection_timeout, Duration::from_secs(60));
        assert_eq!(config.transfer_timeout, Duration::from_secs(90));
        assert_eq!(config.user_agent, "reearth-flow-http-caller/1.0");
        assert_eq!(config.max_redirects, 10);
    }

    #[test]
    fn test_mock_client_captures_requests() {
        let mock = MockHttpClient::with_success("test response".to_string(), 200);
        let request = HttpRequest {
            url: "https://example.com".to_string(),
            method: HttpMethod::GET,
            headers: vec![("Accept".to_string(), "application/json".to_string())],
            query_params: vec![("page".to_string(), "1".to_string())],
            body: None,
            content_type: None,
        };

        let result = mock.execute(&request);
        assert!(result.is_ok());

        let captured = mock.get_requests();
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].url, "https://example.com");
    }

    #[test]
    fn test_mock_client_returns_error() {
        let mock = MockHttpClient::with_error(HttpProcessorError::Request(
            "Connection refused".to_string(),
        ));
        let request = HttpRequest {
            url: "https://example.com".to_string(),
            method: HttpMethod::GET,
            headers: vec![],
            query_params: vec![],
            body: None,
            content_type: None,
        };

        let result = mock.execute(&request);
        assert!(result.is_err());
    }
}
