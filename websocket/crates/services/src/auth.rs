use http::{uri::InvalidUri, Uri};
use reqwest::Response;
use std::{str::FromStr, time::Duration};
use tower::timeout;

use thiserror::Error;

#[derive(Clone, Debug)]
pub struct AuthServiceClient {
    pub uri: Uri,
    pub client: reqwest::Client,
}

#[derive(Error, Debug)]
pub enum AuthServiceError {
    #[error("Invalid URI: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
}

impl AuthServiceClient {
    pub fn new(uri: &str) -> Result<Self, AuthServiceError> {
        let uri: Uri = Uri::from_str(uri)?;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;
        Ok(Self { uri, client })
    }

    pub async fn forward_request(&self, token: &str) -> Result<Response, AuthServiceError> {
        Ok(self
            .client
            .get(self.uri.to_string())
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_authenticate() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(header("Authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = AuthServiceClient::new(&mock_server.uri()).unwrap();
        let response = client.forward_request("test_token").await.unwrap();

        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn test_authenticate_unauthorized() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(header("Authorization", "Bearer invalid_token"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let client = AuthServiceClient::new(&mock_server.uri()).unwrap();
        let response = client.forward_request("invalid_token").await.unwrap();

        assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_authenticate_timeout() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(10)))
            .mount(&mock_server)
            .await;

        let client = AuthServiceClient::new(&mock_server.uri()).unwrap();

        let result = client.forward_request("test_token").await;

        assert!(result.is_err());
    }
}
