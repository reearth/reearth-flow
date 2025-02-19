# Re:Earth Flow API

## Development

### Install toolchains
- Golang (stable)


## Usage

### Start DB
```console
$ make run-db
```

### Run Server
```console
$ go run ./cmd/reearth-flow
```

## Fetch Real-time Logs of the Worker
1. Prepare a network, GCS, Pub/Sub and Redis according to the `server/log_subscriber` README.
2. Execute a sample workflow in the Worker.
3. Fetch the logs via the API.
```
 curl -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "query": "query GetLogs($since: DateTime!, $jobId: ID!) { logs(since: $since, jobId: $jobId) { jobId nodeId timestamp logLevel message } }",
    "variables": {
      "since": "2023-01-01T00:00:00Z",
      "jobId": "5566c900-9581-4c5c-be02-fd13e4d93669"
    }
  }' \
  http://localhost:8080/api/graphql
```
```
{
  "data": {
    "logs": [
      {
        "jobId": "5566c900-9581-4c5c-be02-fd13e4d93669",
        "nodeId": null,
        "timestamp": "2025-02-19T10:00:37.307862Z",
        "logLevel": "INFO",
        "message": "\"FileWriter\" sink start..."
      },
      {
        "jobId": "5566c900-9581-4c5c-be02-fd13e4d93669",
        "nodeId": null,
        "timestamp": "2025-02-19T10:00:37.765254Z",
        "logLevel": "INFO",
        "message": "\"FileWriter\" sink finish. elapsed = 1.899333ms"
      },
      {
        "jobId": "5566c900-9581-4c5c-be02-fd13e4d93669",
        "nodeId": null,
        "timestamp": "2025-02-19T10:00:37.437839Z",
        "logLevel": "INFO",
        "message": "\"FeatureCreator\" finish source complete. elapsed = 1.070708ms"
      }
    ]
  }
}
```