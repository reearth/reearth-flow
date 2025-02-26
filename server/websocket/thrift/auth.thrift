namespace go proto
namespace rs thrift

struct APITokenVerifyRequest {
  1: string token
}

struct APITokenVerifyResponse {
  1: bool authorized
}

service AuthService {
  APITokenVerifyResponse VerifyAPIToken(1: APITokenVerifyRequest request)
} 