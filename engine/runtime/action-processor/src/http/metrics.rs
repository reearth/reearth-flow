use std::time::{Duration, Instant};

use reearth_flow_types::{Attribute, AttributeValue};

use super::client::HttpResponse;
use super::params::ObservabilityConfig;
use super::retry::RetryContext;

pub(crate) struct RequestMetrics {
    pub duration: Duration,
    pub final_url: Option<String>,
    pub retry_count: u32,
    pub bytes_transferred: usize,
}

impl RequestMetrics {
    pub fn new(duration: Duration, response: &HttpResponse, retry_ctx: &RetryContext) -> Self {
        Self {
            duration,
            final_url: None,
            retry_count: retry_ctx.total_attempts.saturating_sub(1),
            bytes_transferred: response.body.len(),
        }
    }

    pub fn add_to_attributes(
        &self,
        attributes: &mut indexmap::IndexMap<Attribute, AttributeValue>,
        config: &Option<ObservabilityConfig>,
    ) {
        let config = match config {
            Some(c) => c,
            None => return,
        };

        if config.track_duration {
            let attr_name = config
                .duration_attribute
                .clone()
                .unwrap_or_else(|| "_request_duration_ms".to_string());
            attributes.insert(
                Attribute::new(attr_name),
                AttributeValue::Number(serde_json::Number::from(self.duration.as_millis() as i64)),
            );
        }

        if config.track_final_url {
            if let Some(final_url) = &self.final_url {
                let attr_name = config
                    .final_url_attribute
                    .clone()
                    .unwrap_or_else(|| "_final_url".to_string());
                attributes.insert(
                    Attribute::new(attr_name),
                    AttributeValue::String(final_url.clone()),
                );
            }
        }

        if config.track_retry_count {
            let attr_name = config
                .retry_count_attribute
                .clone()
                .unwrap_or_else(|| "_retry_count".to_string());
            attributes.insert(
                Attribute::new(attr_name),
                AttributeValue::Number(serde_json::Number::from(self.retry_count)),
            );
        }

        if config.track_bytes {
            let attr_name = config
                .bytes_attribute
                .clone()
                .unwrap_or_else(|| "_bytes_transferred".to_string());
            attributes.insert(
                Attribute::new(attr_name),
                AttributeValue::Number(serde_json::Number::from(self.bytes_transferred)),
            );
        }
    }
}

pub(crate) struct RequestTimer {
    start: Instant,
}

impl RequestTimer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::thread;

    #[test]
    fn test_metrics_creation() {
        let response = HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: "test response".to_string(),
        };

        let retry_ctx = RetryContext {
            attempt: 2,
            total_attempts: 3,
        };

        let metrics = RequestMetrics::new(Duration::from_millis(150), &response, &retry_ctx);

        assert_eq!(metrics.duration, Duration::from_millis(150));
        assert_eq!(metrics.retry_count, 2);
        assert_eq!(metrics.bytes_transferred, "test response".len());
    }

    #[test]
    fn test_add_duration_to_attributes() {
        let response = HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: "test".to_string(),
        };

        let retry_ctx = RetryContext {
            attempt: 0,
            total_attempts: 1,
        };

        let metrics = RequestMetrics::new(Duration::from_millis(500), &response, &retry_ctx);

        let config = ObservabilityConfig {
            track_duration: true,
            duration_attribute: Some("_duration".to_string()),
            track_final_url: false,
            final_url_attribute: None,
            track_retry_count: false,
            retry_count_attribute: None,
            track_bytes: false,
            bytes_attribute: None,
        };

        let mut attributes = indexmap::IndexMap::new();
        metrics.add_to_attributes(&mut attributes, &Some(config));

        assert!(attributes.contains_key(&Attribute::new("_duration".to_string())));
    }

    #[test]
    fn test_add_all_metrics() {
        let response = HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: "x".repeat(1000),
        };

        let retry_ctx = RetryContext {
            attempt: 2,
            total_attempts: 3,
        };

        let metrics = RequestMetrics::new(Duration::from_millis(750), &response, &retry_ctx);

        let config = ObservabilityConfig {
            track_duration: true,
            duration_attribute: None,
            track_final_url: false,
            final_url_attribute: None,
            track_retry_count: true,
            retry_count_attribute: None,
            track_bytes: true,
            bytes_attribute: None,
        };

        let mut attributes = indexmap::IndexMap::new();
        metrics.add_to_attributes(&mut attributes, &Some(config));

        assert!(attributes.contains_key(&Attribute::new("_request_duration_ms".to_string())));
        assert!(attributes.contains_key(&Attribute::new("_retry_count".to_string())));
        assert!(attributes.contains_key(&Attribute::new("_bytes_transferred".to_string())));
    }

    #[test]
    fn test_timer() {
        let timer = RequestTimer::new();
        thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
    }
}
