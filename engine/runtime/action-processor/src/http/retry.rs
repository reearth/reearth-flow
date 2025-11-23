use std::thread;
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Method;

use super::body::BodyContent;
use super::client::{HttpClient, HttpResponse};
use super::errors::{HttpProcessorError, Result};
use super::params::RetryConfig;

pub(crate) struct RetryContext {
    pub attempt: u32,
    pub total_attempts: u32,
}

pub(crate) fn execute_with_retry(
    client: &dyn HttpClient,
    method: Method,
    url: String,
    headers: HeaderMap,
    query_params: Vec<(String, String)>,
    body: Option<BodyContent>,
    retry_config: &Option<RetryConfig>,
) -> Result<(HttpResponse, RetryContext)> {
    let config = retry_config.as_ref();
    let max_attempts = config.map(|c| c.max_attempts).unwrap_or(1);

    let mut retry_ctx = RetryContext {
        attempt: 0,
        total_attempts: 0,
    };

    for attempt in 0..max_attempts {
        retry_ctx.attempt = attempt;

        let method_clone = method.clone();
        let headers_clone = headers.clone();
        let query_clone = query_params.clone();
        let body_clone = clone_body_content(&body);

        match client.send_request(method_clone, &url, headers_clone, query_clone, body_clone) {
            Ok(response) => {
                retry_ctx.total_attempts = attempt + 1;

                if let Some(cfg) = config {
                    if should_retry_status(response.status_code, cfg) && attempt + 1 < max_attempts
                    {
                        let mut header_map = HeaderMap::new();
                        for (k, v) in &response.headers {
                            if let (Ok(name), Ok(value)) =
                                (k.parse::<HeaderName>(), v.parse::<HeaderValue>())
                            {
                                header_map.insert(name, value);
                            }
                        }
                        let delay = calculate_backoff_delay(cfg, attempt, &header_map);
                        thread::sleep(delay);
                        continue;
                    }
                }

                return Ok((response, retry_ctx));
            }
            Err(e) => {
                retry_ctx.total_attempts = attempt + 1;

                if attempt + 1 >= max_attempts {
                    return Err(e);
                }

                if let Some(cfg) = config {
                    let delay = calculate_backoff_delay(cfg, attempt, &HeaderMap::new());
                    thread::sleep(delay);
                }
            }
        }
    }

    Err(HttpProcessorError::Request(
        "Max retry attempts exceeded".to_string(),
    ))
}

fn clone_body_content(body: &Option<BodyContent>) -> Option<BodyContent> {
    body.as_ref().map(|b| match b {
        BodyContent::Text(s) => BodyContent::Text(s.clone()),
        BodyContent::Binary(b) => BodyContent::Binary(b.clone()),
        BodyContent::Form(f) => BodyContent::Form(f.clone()),
        BodyContent::Multipart(_) => {
            panic!("Retry is not supported for multipart requests")
        }
    })
}

fn should_retry_status(status: u16, config: &RetryConfig) -> bool {
    if let Some(retry_statuses) = &config.retry_on_status {
        retry_statuses.contains(&status)
    } else {
        (500..600).contains(&status)
    }
}

fn calculate_backoff_delay(config: &RetryConfig, attempt: u32, headers: &HeaderMap) -> Duration {
    // Check for Retry-After header
    if config.honor_retry_after {
        if let Some(retry_after) = parse_retry_after_header(headers) {
            return retry_after;
        }
    }

    let delay_ms = config.initial_delay_ms as f64 * config.backoff_multiplier.powi(attempt as i32);
    let delay_ms = delay_ms.min(config.max_delay_ms as f64) as u64;

    Duration::from_millis(delay_ms)
}

fn parse_retry_after_header(headers: &HeaderMap) -> Option<Duration> {
    let retry_after = headers.get("retry-after")?.to_str().ok()?;

    if let Ok(seconds) = retry_after.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_retry_5xx() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 100,
            backoff_multiplier: 2.0,
            max_delay_ms: 10000,
            retry_on_status: None,
            honor_retry_after: true,
        };

        assert!(should_retry_status(500, &config));
        assert!(should_retry_status(502, &config));
        assert!(should_retry_status(503, &config));
        assert!(!should_retry_status(200, &config));
        assert!(!should_retry_status(404, &config));
    }

    #[test]
    fn test_should_retry_custom_status() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 100,
            backoff_multiplier: 2.0,
            max_delay_ms: 10000,
            retry_on_status: Some(vec![429, 503]),
            honor_retry_after: true,
        };

        assert!(should_retry_status(429, &config));
        assert!(should_retry_status(503, &config));
        assert!(!should_retry_status(500, &config));
        assert!(!should_retry_status(404, &config));
    }

    #[test]
    fn test_backoff_delay() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay_ms: 100,
            backoff_multiplier: 2.0,
            max_delay_ms: 1000,
            retry_on_status: None,
            honor_retry_after: true,
        };

        let headers = HeaderMap::new();

        // Attempt 0: 100ms
        let delay = calculate_backoff_delay(&config, 0, &headers);
        assert_eq!(delay, Duration::from_millis(100));

        // Attempt 1: 200ms
        let delay = calculate_backoff_delay(&config, 1, &headers);
        assert_eq!(delay, Duration::from_millis(200));

        // Attempt 2: 400ms
        let delay = calculate_backoff_delay(&config, 2, &headers);
        assert_eq!(delay, Duration::from_millis(400));

        // Attempt 3: 800ms
        let delay = calculate_backoff_delay(&config, 3, &headers);
        assert_eq!(delay, Duration::from_millis(800));

        // Attempt 4: would be 1600ms but capped at max_delay_ms (1000ms)
        let delay = calculate_backoff_delay(&config, 4, &headers);
        assert_eq!(delay, Duration::from_millis(1000));
    }

    #[test]
    fn test_parse_retry_after() {
        let mut headers = HeaderMap::new();
        headers.insert("retry-after", "5".parse().unwrap());

        let delay = parse_retry_after_header(&headers);
        assert_eq!(delay, Some(Duration::from_secs(5)));
    }
}
