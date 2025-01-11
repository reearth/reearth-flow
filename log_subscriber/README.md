# Log Subscriber

The log_subscriber is a Go application that subscribes to a Google Cloud Pub/Sub topic to pull and process workflow execution logs. Once a message (log event) is received, the application writes the log into both Redis and GCS. If either write fails, the subscriber lets Pub/Sub retry automatically.

## Overview

This subscriber listens to Pub/Sub messages containing workflow logs. By storing these logs in both Redis and GCS, users can view real-time logs via a web interface and later retrieve complete execution logs from long-term storage (GCS).

A simplified diagram might look like this
```
+-----------------+     +-------------------+         +-----------+
| reearth-flow-   |     | Pub/Sub Topic     |         | log_sub-  |
| worker (Rust)   | --> | (flow-log-stream) |  -->    | scriber   |
| (workflow run)  |     |                   |         | (Go)      |
+-----------------+     +-------------------+         +-----------+
                                                       /       \
                                                      /         \
                                           (write) Redis       GCS (write)
```

## Features & User Story
### Real-time Monitoring
Users can watch the workflow’s execution status and logs in real time on a web-based dashboard.
### Centralized Log Storage
Logs are stored both in Redis (for quick real-time access) and in GCS (for longer retention and detailed inspection).
### Automatic Retry
If any write (Redis/GCS) fails, the system relies on Pub/Sub’s retry mechanism to re-deliver the message until it’s successfully processed.

## Specifications

### Log Schema

Each log message has the following schema:
```

{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "LogStreamEvent",
  "type": "object",
  "required": ["jobId","logLevel","message","timestamp","workflowId"],
  "properties": {
    "jobId": {
      "type": "string",
      "format": "uuid"
    },
    "logLevel": {
      "type": "string",
      "enum": ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"]
    },
    "message": {
      "type": "string"
    },
    "nodeId": {
      "type": ["string","null"]
    },
    "timestamp": {
      "type": "string",
      "format": "date-time"
    },
    "workflowId": {
      "type": "string",
      "format": "uuid"
    }
  }
}
```

### Redis

Each log entry is stored as a key-value in Redis. It is assumed that the TTL is set to a short time.

**Key format**
```
log:{workflowId}:{jobId}:{timestamp}
```


**Example**
```
log:00caad2a-9f7d-4189-b479-153fa9ea36dc:5566c900-9581-4c5c-be02-fd13e4d93669:2025-01-11T09:12:54.943837Z
```

### GCS

The same log entry is also stored as a JSON file on GCS

**Path format**
```
gs://<your-bucket-name>/artifacts/logs/{yyyy}/{MM}/{dd}/{workflowId}/{jobId}/{timestamp}.json
```
**Example**
```
gs://reearth-flow-oss-bucket/artifacts/logs/2025/01/11/00caad2a-9f7d-4189-b479-153fa9ea36dc/5566c900-9581-4c5c-be02-fd13e4d93669/2025-01-11T09:12:54.943837Z.json
```
### Retry Behavior

Pub/Sub provides automatic retry. The subscriber logic is
1.	Write to Redis
2.	Write to GCS
3.	If both succeed, `m.Ack();` otherwise `m.Nack()` and let Pub/Sub retry


**Note**

This can lead to duplicate entries in Redis if the first write was successful but the second failed.
Idempotency is not handled in this approach.

## Usage
Create a network
```
docker network create reearth-flow-net
```

### GCS & Pub/Sub Setup


```
cd engine
docker compose build
docker compose up -d
```

- Bucket: reearth-flow-oss-bucket
- Topic IDs: flow-edge-pass-through-topic, flow-log-stream-topic, flow-job-complete-topic
- Subscription IDs: flow-edge-pass-through-topic-sub, flow-log-stream-topic-sub, flow-job-complete-topic-sub

### Redis & Subscriber Setup
The log_subscriber uses the following environment variables

| Name                                  | Description                                             | Default                     |
| ------------------------------------- | ------------------------------------------------------- | --------------------------- |
| `PUBSUB_EMULATOR_HOST`                | Pub/Sub emulator endpoint                               | `""`                        |
| `FLOW_LOG_SUBSCRIBER_PROJECT_ID`      | GCP project ID when connecting to Pub/Sub               | `local-project`             |
| `FLOW_LOG_SUBSCRIBER_SUBSCRIPTION_ID` | The Pub/Sub subscription ID to use for the subscription | `flow-log-stream-topic-sub` |
| `FLOW_LOG_SUBSCRIBER_REDIS_ADDR`      | The Redis address to connect to (in host:port format)   | `localhost:6379`            |
| `FLOW_LOG_SUBSCRIBER_REDIS_PASSWORD`  | Redis password                                          | `""`                        |
| `FLOW_LOG_SUBSCRIBER_GCS_BUCKET_NAME` | The name of the GCS bucket to write the logs to         | `reearth-flow-oss-bucket`   |
| `STORAGE_EMULATOR_HOST`               | GCS emulator endpoint                                   | `""`                        |


```
cd log_subscriber
docker compose build
docker compose up -d
```

### Prepare Example Workflow

Below is an example demonstrating how to run a workflow (e.g., using cargo run) and retrieve logs via Pub/Sub.

**Workflow (example.yml)**
```
# yaml-language-server: $schema=https://raw.githubusercontent.com/reearth/reearth-flow/main/engine/schema/workflow.json
id: 00caad2a-9f7d-4189-b479-153fa9ea36dc
name: "Example"
entryGraphId: 3e3450c8-2344-4728-afa9-5fdb81eec33a
with:
  outputPath:
graphs:
  - id: 3e3450c8-2344-4728-afa9-5fdb81eec33a
    name: entry_point
    nodes:
      - id: 90f40a3e-61d3-48e2-a328-e7226c2ad1ae
        name: FeatureCreator
        type: action
        action: FeatureCreator
        with:
          creator: |
            [
              #{
                country: "Japan",
                city: "Tokyo",
                population: 37977000,
              },
              #{
                city: "Osaka",
                population: 14977000,
                country: "Japan",
              }
            ]

      - id: f5e66920-24c0-4c70-ae16-6be1ed3b906c
        name: FileWriter
        type: action
        action: FileWriter
        with:
          format: json
          output: |
            file::join_path(env.get("workerArtifactPath"), env.get("outputPath"))

    edges:
      - id: c064cf52-705f-443a-b2de-6795266c540d
        from: 90f40a3e-61d3-48e2-a328-e7226c2ad1ae
        to: f5e66920-24c0-4c70-ae16-6be1ed3b906c
        fromPort: default
        toPort: default
```
**Upload to the bucket**
```
curl -X POST \
  --data-binary @example.yml \
  "http://localhost:4443/upload/storage/v1/b/reearth-flow-oss-bucket/o?uploadType=media&name=workflows/example.yml"
```

**Metadata (metadata.json)**
```

{
  "jobId": "5566c900-9581-4c5c-be02-fd13e4d93669",
  "assets": {
    "baseUrl": "gs://reearth-flow-oss-bucket/assets",
    "files": []
  },
  "artifactBaseUrl": "gs://reearth-flow-oss-bucket/artifacts",
  "timestamps": {
    "created": "2024-10-29T03:55:00Z"
  }
}
```
**Upload to the bucket**
```
curl -X POST \
  --data-binary @metadata.json \
  "http://localhost:4443/upload/storage/v1/b/reearth-flow-oss-bucket/o?uploadType=media&name=metadata/metadata.json"
```

### Run the workflow
```
cd engine
# For the emulators
export STORAGE_EMULATOR_HOST=http://localhost:4443
export PUBSUB_EMULATOR_HOST=0.0.0.0:8085

# Run the workflow
cargo run --package reearth-flow-worker \
  -- --workflow gs://reearth-flow-oss-bucket/workflows/example.yml \
  --metadata-path gs://reearth-flow-oss-bucket/metadata/metadata.json \
  --var='outputPath=result.json'
```
### Confirm Logs in Redis
```
docker exec -it log-subscriber-redis redis-cli
127.0.0.1:6379> KEYS *
 1) "log:00caad2a-9f7d-4189-b479-153fa9ea36dc:5566c900-9581-4c5c-be02-fd13e4d93669:2025-01-11T09:12:54.943837Z"
 2) "log:00caad2a-9f7d-4189-b479-153fa9ea36dc:5566c900-9581-4c5c-be02-fd13e4d93669:2025-01-11T09:12:54.602634Z"
 3) "log:00caad2a-9f7d-4189-b479-153fa9ea36dc:5566c900-9581-4c5c-be02-fd13e4d93669:2025-01-11T09:12:54.487779Z"
127.0.0.1:6379> get "log:00caad2a-9f7d-4189-b479-153fa9ea36dc:5566c900-9581-4c5c-be02-fd13e4d93669:2025-01-11T09:12:54.487779Z"
```
**Example Output**
```
"{\"workflowId\":\"00caad2a-9f7d-4189-b479-153fa9ea36dc\",\"jobId\":\"5566c900-9581-4c5c-be02-fd13e4d93669\",\"nodeId\":\"f5e66920-24c0-4c70-ae16-6be1ed3b906c\",\"logLevel\":\"INFO\",\"timestamp\":\"2025-01-11T09:12:54.487779Z\",\"message\":\"\\\"FileWriter\\\" sink start...\"}"
"{\"workflowId\":\"00caad2a-9f7d-4189-b479-153fa9ea36dc\",\"jobId\":\"5566c900-9581-4c5c-be02-fd13e4d93669\",\"nodeId\":\"\",\"logLevel\":\"INFO\",\"timestamp\":\"2025-01-11T09:12:54.602634Z\",\"message\":\"\\\"FeatureCreator\\\" finish source complete. elapsed = 855.334\xc2\xb5s\"}"
"{\"workflowId\":\"00caad2a-9f7d-4189-b479-153fa9ea36dc\",\"jobId\":\"5566c900-9581-4c5c-be02-fd13e4d93669\",\"nodeId\":\"f5e66920-24c0-4c70-ae16-6be1ed3b906c\",\"logLevel\":\"INFO\",\"timestamp\":\"2025-01-11T09:12:54.943837Z\",\"message\":\"\\\"FileWriter\\\" sink finish. elapsed = 1.688292ms\"}"
```
### Confirm Logs in GCS
```
curl -X GET "http://localhost:4443/storage/v1/b/reearth-flow-oss-bucket/o?prefix=artifacts/logs"
curl -o log.json "http://localhost:4443/download/storage/v1/b/reearth-flow-oss-bucket/o/artifacts%2Flogs%2F2025%2F01%2F11%2F00caad2a-9f7d-4189-b479-153fa9ea36dc%2F5566c900-9581-4c5c-be02-fd13e4d93669%2F2025-01-11T09:12:54.487779Z.json?alt=media"
cat log.json
```
**Example Output**
```
{"workflowId":"00caad2a-9f7d-4189-b479-153fa9ea36dc","jobId":"5566c900-9581-4c5c-be02-fd13e4d93669","nodeId":"f5e66920-24c0-4c70-ae16-6be1ed3b906c","timestamp":"2025-01-11T09:12:54.487779Z","logLevel":"INFO","message":"\"FileWriter\" sink start..."}
{"workflowId":"00caad2a-9f7d-4189-b479-153fa9ea36dc","jobId":"5566c900-9581-4c5c-be02-fd13e4d93669","timestamp":"2025-01-11T09:12:54.602634Z","logLevel":"INFO","message":"\"FeatureCreator\" finish source complete. elapsed = 855.334µs"}
{"workflowId":"00caad2a-9f7d-4189-b479-153fa9ea36dc","jobId":"5566c900-9581-4c5c-be02-fd13e4d93669","nodeId":"f5e66920-24c0-4c70-ae16-6be1ed3b906c","timestamp":"2025-01-11T09:12:54.943837Z","logLevel":"INFO","message":"\"FileWriter\" sink finish. elapsed = 1.688292ms"}
```


### (Optional) Pulling Logs Dilectly
```
curl -X POST "http://localhost:8085/v1/projects/local-project/subscriptions/flow-log-stream-topic-sub:pull" \
     -H "Content-Type: application/json" \
     -d '{
           "maxMessages": 1000
         }'
```