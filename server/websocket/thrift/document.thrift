namespace go proto
namespace rs thrift

struct Document {
  1: string id
  2: list<i32> updates
  3: i32 version
  4: string timestamp
}

struct History {
  1: list<i32> updates
  2: i32 version
  3: string timestamp
}

struct GetLatestRequest {
  1: string doc_id
}

struct GetLatestResponse {
  1: Document document
}

struct GetHistoryRequest {
  1: string doc_id
}

struct GetHistoryResponse {
  1: list<History> history
}

struct RollbackRequest {
  1: string doc_id
  2: i32 version
}

struct RollbackResponse {
  1: Document document
}

service DocumentService {
  GetLatestResponse GetLatest(1: GetLatestRequest request)
  GetHistoryResponse GetHistory(1: GetHistoryRequest request)
  RollbackResponse Rollback(1: RollbackRequest request)
} 