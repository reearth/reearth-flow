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

## Test GraphQL 
### jobResolver.Logs()
1. Prepare a network, GCS, Pub/Sub and Redis according to the `server/subscriber` README.
2. Change the jobId in `metadata.json` to the appropriate one.
3. Execute a sample workflow in the Worker.
4. Add new variables to `.env` and run the API
5. Fetch the logs via the API.
```
$ curl -X POST "http://localhost:8080/api/graphql" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer XXX" \
  -d '{
    "query": "query GetJobLogs($id: ID!, $since: DateTime!) { job(id: $id) { logs(since: $since) { timestamp message } } }",
    "variables": {
      "id": "2f0307f1-e41f-4952-9b95-37ecb711a5ca",
      "since": "2025-02-28T00:00:00Z"
    }
  }'
```
```
{
  "data": {
    "job": {
      "logs": [
        {
          "timestamp": "2025-02-28T13:03:15.573031Z",
          "message": "\"FeatureCreator\" finish source complete. elapsed = 796.459µs"
        },
        {
          "timestamp": "2025-02-28T11:51:06.162139Z",
          "message": "\"FileWriter\" sink start..."
        },
        {
          "timestamp": "2025-02-28T11:51:06.718532Z",
          "message": "\"FileWriter\" sink finish. elapsed = 1.959792ms"
        }
      ]
    }
  }
}
```
### subscriptionResolver().Logs()
1. Prepare a network, GCS, Pub/Sub and Redis according to the `server/subscriber` README.
2. Add new variables to `.env` and run the API
3. Establish a subscription using WebSocket.
```
wscat -c ws://localhost:8080/api/graphql   
```
```
{
  "type": "connection_init",
  "payload": {
    "Authorization": "Bearer XXX"
  }
}
```
```
{
  "id": "1",
  "type": "start",
  "payload": {
    "query": "subscription logs($jobId: ID!) { logs(jobId: $jobId) { jobId nodeId timestamp logLevel message } }",
    "variables": {
      "jobId": "2f0307f1-e41f-4952-9b95-37ecb711a5ca"
    }
  }
}
```
4. Change the jobId in `metadata.json` to the appropriate one.
5. Execute a sample workflow in the Worker.
6. Check subscribed logs in WebSocket.
```
< {"type":"ka"}
< {"payload":{"data":{"logs":{"jobId":"2f0307f1-e41f-4952-9b95-37ecb711a5ca","nodeId":null,"timestamp":"2025-02-28T15:11:28.780651Z","logLevel":"INFO","message":"\"FileWriter\" sink finish. elapsed = 1.537917ms"}}},"id":"1","type":"data"}
< {"payload":{"data":{"logs":{"jobId":"2f0307f1-e41f-4952-9b95-37ecb711a5ca","nodeId":null,"timestamp":"2025-02-28T15:11:28.465097Z","logLevel":"INFO","message":"\"FeatureCreator\" finish source complete. elapsed = 632.125µs"}}},"id":"1","type":"data"}
< {"payload":{"data":{"logs":{"jobId":"2f0307f1-e41f-4952-9b95-37ecb711a5ca","nodeId":null,"timestamp":"2025-02-28T15:11:28.282845Z","logLevel":"INFO","message":"\"FileWriter\" sink start..."}}},"id":"1","type":"data"}
< {"type":"ka"}
```

## Authorization System

Re:Earth Flow uses Role-Based Access Control (RBAC) to manage permissions. The system is built using Cerbos for policy enforcement.

### Authorization Configuration
All authorization-related definitions are managed in `api/internal/rbac/definitions.go`. This file contains:

- Resource definitions (e.g., project, workflow)
- Action definitions (e.g., read, edit)
- Role definitions (owner, maintainer, writer, reader)
- Permission mappings between resources, actions, and roles

To add or modify permissions:
1. Open `api/internal/rbac/definitions.go`
2. Add/modify resources in the `ResourceXXX` constants
3. Add/modify actions in the `ActionXXX` constants
4. Add/modify roles in the `roleXXX` constants
5. Update the permission mappings in the `DefineResources` function

### Deployment
The permission definitions are automatically synchronized with the Cerbos server's storage bucket when changes are merged into the main branch via CI. This ensures that the latest permission settings are always available for authorization checks.

### Implementation in Use Cases
Permission checks should be implemented in use case interactors.
