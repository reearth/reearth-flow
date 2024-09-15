use super::error::JwtError;
use cached::{Cached, TimedCache};
use http::StatusCode;
use services::AuthServiceClient;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct JwtValidator {
    client: AuthServiceClient,
    cache: Arc<Mutex<TimedCache<String, bool>>>,
}

impl JwtValidator {
    pub fn new(client: AuthServiceClient, cache_duration: Duration) -> Self {
        Self {
            client,
            cache: Arc::new(Mutex::new(TimedCache::with_lifespan(
                cache_duration.as_secs(),
            ))),
        }
    }

    pub async fn verify(&self, token: &str) -> Result<bool, JwtError> {
        // first check cache
        {
            let mut cache = self.cache.lock().await;
            if let Some(&cached_result) = cache.cache_get(token) {
                return Ok(cached_result);
            }
        }

        // if cache not hit, send request
        let response = self.client.forward_request(token).await?;
        let result = response.status() == StatusCode::OK;

        // save result to cache
        {
            let mut cache = self.cache.lock().await;
            cache.cache_set(token.to_string(), result);
        }

        if result {
            Ok(true)
        } else {
            Err(JwtError::VerificationFailed(response.status()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_jwt_verify_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/"))
            .and(header("Authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = AuthServiceClient::new(&mock_server.uri()).unwrap();
        let jwt = JwtValidator::new(client, Duration::from_secs(300)); // 5分钟缓存

        let result1 = jwt.verify("test_token").await;
        assert!(result1.is_ok());
        assert!(result1.unwrap());

        let result2 = jwt.verify("test_token").await;
        assert!(result2.is_ok());
        assert!(result2.unwrap());
    }

    #[tokio::test]
    async fn test_jwt_verify_failure() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/"))
            .and(header("Authorization", "Bearer invalid_token"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let client = AuthServiceClient::new(&mock_server.uri()).unwrap();
        let jwt = JwtValidator::new(client, Duration::from_secs(300));

        let result = jwt.verify("invalid_token").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            JwtError::VerificationFailed(_)
        ));
    }
}
