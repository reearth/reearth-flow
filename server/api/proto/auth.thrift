namespace go proto

struct APITokenVerifyRequest {
  1: string token
}

struct APITokenVerifyResponse {
  1: bool authorized
}

service AuthService {
  // Verify API token
  APITokenVerifyResponse VerifyAPIToken(1: APITokenVerifyRequest request)
} 