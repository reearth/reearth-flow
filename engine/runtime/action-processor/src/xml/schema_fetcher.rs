use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::errors::{Result, XmlProcessorError};

/// Trait for fetching schemas from various sources
pub(crate) trait SchemaFetcher: Send + Sync {
    /// Fetch schema content from the given URL
    fn fetch_schema(&self, url: &str) -> Result<String>;
}

/// HTTP/HTTPS schema fetcher implementation
#[derive(Clone)]
pub(crate) struct HttpSchemaFetcher {
    client: Arc<Mutex<Option<reqwest::blocking::Client>>>,
    max_retries: usize,
    retry_delay: Duration,
}

impl HttpSchemaFetcher {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            max_retries: 2,
            retry_delay: Duration::from_millis(0),
        }
    }

    #[cfg(test)]
    pub fn with_retry_config(mut self, max_retries: usize, retry_delay: Duration) -> Self {
        self.max_retries = max_retries;
        self.retry_delay = retry_delay;
        self
    }

    fn get_or_create_client(&self) -> Result<reqwest::blocking::Client> {
        let mut guard = self.client.lock().unwrap();
        if let Some(client) = guard.as_ref() {
            return Ok(client.clone());
        }

        // Create client outside of async context (in processor's thread)
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(10))
            .user_agent("reearth-flow-xml-validator/1.0")
            .build()
            .map_err(|e| {
                XmlProcessorError::Validator(format!("Failed to create HTTP client: {e}"))
            })?;

        *guard = Some(client.clone());
        Ok(client)
    }

    fn fetch_with_retry(&self, url: &str) -> Result<String> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match self.try_fetch(url) {
                Ok(content) => return Ok(content),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries {
                        std::thread::sleep(self.retry_delay);
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            XmlProcessorError::Validator("Unknown error during schema fetch".to_string())
        }))
    }

    fn try_fetch(&self, url: &str) -> Result<String> {
        let client = self.get_or_create_client()?;
        let response = client.get(url).send().map_err(|e| {
            XmlProcessorError::Validator(format!(
                "Failed to fetch HTTP/HTTPS schema from {url}: {e}"
            ))
        })?;

        // Check for successful status codes (2xx)
        if !response.status().is_success() {
            return Err(XmlProcessorError::Validator(format!(
                "HTTP error {} when fetching schema from {}",
                response.status(),
                url
            )));
        }

        let content = response.text().map_err(|e| {
            XmlProcessorError::Validator(format!("Failed to read schema content from {url}: {e}"))
        })?;

        Ok(content)
    }
}

impl SchemaFetcher for HttpSchemaFetcher {
    fn fetch_schema(&self, url: &str) -> Result<String> {
        self.fetch_with_retry(url)
    }
}

impl Default for HttpSchemaFetcher {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP/HTTPS schema fetcher with bundled schema fallback
#[derive(Clone)]
pub(crate) struct HttpSchemaFetcherWithFallback {
    inner: HttpSchemaFetcher,
}

impl HttpSchemaFetcherWithFallback {
    pub fn new() -> Self {
        Self {
            inner: HttpSchemaFetcher::new(),
        }
    }

    #[cfg(test)]
    pub fn with_retry_config(mut self, max_retries: usize, retry_delay: Duration) -> Self {
        self.inner = self.inner.with_retry_config(max_retries, retry_delay);
        self
    }
}

impl SchemaFetcher for HttpSchemaFetcherWithFallback {
    fn fetch_schema(&self, url: &str) -> Result<String> {
        match self.inner.fetch_with_retry(url) {
            Ok(content) => Ok(content),
            Err(e) => {
                // Fallback to bundled schema if HTTP fails
                if let Some(bundled_content) = super::bundled_schemas::get(url) {
                    Ok(bundled_content.to_string())
                } else {
                    Err(e)
                }
            }
        }
    }
}

impl Default for HttpSchemaFetcherWithFallback {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock implementation for testing
#[cfg(test)]
#[derive(Clone)]
pub(crate) struct MockSchemaFetcher {
    pub responses: std::collections::HashMap<String, Result<String>>,
    pub call_count: Arc<parking_lot::RwLock<std::collections::HashMap<String, usize>>>,
}

#[cfg(test)]
impl MockSchemaFetcher {
    pub fn new() -> Self {
        MockSchemaFetcher {
            responses: std::collections::HashMap::new(),
            call_count: Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub fn with_response(mut self, url: &str, response: Result<String>) -> Self {
        self.responses.insert(url.to_string(), response);
        self
    }

    pub fn get_call_count(&self, url: &str) -> usize {
        *self.call_count.read().get(url).unwrap_or(&0)
    }
}

#[cfg(test)]
impl SchemaFetcher for MockSchemaFetcher {
    fn fetch_schema(&self, url: &str) -> Result<String> {
        // Track call count
        {
            let mut count = self.call_count.write();
            *count.entry(url.to_string()).or_insert(0) += 1;
        }

        self.responses.get(url).cloned().unwrap_or_else(|| {
            Err(XmlProcessorError::Validator(format!(
                "No mock response for URL: {url}"
            )))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_schema_fetcher_creation() {
        let fetcher = HttpSchemaFetcher::new();
        assert_eq!(fetcher.max_retries, 2);
        assert_eq!(fetcher.retry_delay, Duration::from_millis(0));
    }

    #[test]
    fn test_http_schema_fetcher_with_retry_config() {
        let fetcher = HttpSchemaFetcher::new().with_retry_config(5, Duration::from_millis(500));
        assert_eq!(fetcher.max_retries, 5);
        assert_eq!(fetcher.retry_delay, Duration::from_millis(500));
    }

    #[test]
    fn test_mock_schema_fetcher() {
        let mock = MockSchemaFetcher::new()
            .with_response(
                "http://example.com/schema.xsd",
                Ok("test schema".to_string()),
            )
            .with_response(
                "http://error.com/schema.xsd",
                Err(XmlProcessorError::Validator("Error".to_string())),
            );

        // Test successful fetch
        let result = mock.fetch_schema("http://example.com/schema.xsd");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test schema");
        assert_eq!(mock.get_call_count("http://example.com/schema.xsd"), 1);

        // Test error fetch
        let result = mock.fetch_schema("http://error.com/schema.xsd");
        assert!(result.is_err());
        assert_eq!(mock.get_call_count("http://error.com/schema.xsd"), 1);

        // Test non-existent URL
        let result = mock.fetch_schema("http://notfound.com/schema.xsd");
        assert!(result.is_err());
    }
}
