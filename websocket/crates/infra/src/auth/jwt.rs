use super::error::JwtError;
use http::StatusCode;
use services::AuthServiceClient;

pub struct Jwt {
    token: String,
    client: AuthServiceClient,
}

impl Jwt {
    pub fn new(token: String, client: AuthServiceClient) -> Self {
        Self { token, client }
    }
    pub async fn verify(&self) -> Result<bool, JwtError> {
        let response = self.client.forward_request(&self.token).await?;
        // if status code is 200, return true
        if response.status() == StatusCode::OK {
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
            .mount(&mock_server)
            .await;

        let client = AuthServiceClient::new(&mock_server.uri()).unwrap();
        let jwt = Jwt::new("test_token".to_string(), client);

        let result = jwt.verify().await;
        assert!(result.is_ok());
        assert!(result.unwrap());
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
        let jwt = Jwt::new("invalid_token".to_string(), client);

        let result = jwt.verify().await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            JwtError::VerificationFailed(_)
        ));
    }
}
