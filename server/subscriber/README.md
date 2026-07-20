# Log Subscriber

The log_subscriber is a Go application that subscribes to a Google Cloud Pub/Sub topic to pull and process workflow execution logs. Once a message (log event) is received, the application writes the log into Redis. If either write fails, the subscriber lets Pub/Sub retry automatically.

## Overview

This subscriber listens to Pub/Sub messages containing workflow logs. By storing these logs in Redis, users can view real-time logs via a web interface.

A simplified diagram might look like this
```
+-----------------+     +-------------------+         +-----------+
| reearth-flow-   |     | Pub/Sub Topic     |         | log_sub-  |
| worker (Rust)   | --> | (flow-log-stream) |  -->    | scriber   |
| (workflow run)  |     |                   |         | (Go)      |
+-----------------+     +-------------------+         +-----------+
                                                       /
                                                      /
                                           (write) Redis
```

## Features & User Story
### Real-time Monitoring
Users can watch the workflow’s execution status and logs in real time on a web-based dashboard.
### Centralized Log Storage
Logs are stored both in Redis for quick real-time access.
### Automatic Retry
If any write fails, the system relies on Pub/Sub’s retry mechanism to re-deliver the message until it’s successfully processed.

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


### Retry Behavior

Pub/Sub provides automatic retry. The subscriber logic is
1.	Write to Redis
2.	If succeed, `m.Ack();` otherwise `m.Nack()` and let Pub/Sub retry


**Note**

Idempotency is not handled in this approach.
This can lead to duplicate entries in Redis.


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
| `REEARTH_FLOW_SUBSCRIBER_DIAGNOSTIC_SUBSCRIPTION_ID` | The Pub/Sub subscription ID for per-node/job `DiagnosticEvent` ingestion (writes Redis `diagnostics:*` lists and the Mongo `nodeDiagnostics` collection) | `""` (unset — deliberately, unlike the sibling subscription IDs above; see "Diagnostics ingestion (deploy order)" below) |


```
cd server/subscriber
cp .env.example .env
make run
```

### Diagnostics ingestion (deploy order)

The engine publishes structured per-node/job `DiagnosticEvent`s to a Pub/Sub
topic, gated behind two env vars read in
`engine/worker/src/types/diagnostic_event.rs`:

- `FLOW_WORKER_ENABLE_DIAGNOSTICS` — publish gate, defaults to `false` (unset
  or anything other than `"true"` keeps publishing off).
- `FLOW_WORKER_DIAGNOSTIC_TOPIC` — the topic name, defaults to
  `flow-diagnostic-topic` when unset.

On the subscriber side, `REEARTH_FLOW_SUBSCRIBER_DIAGNOSTIC_SUBSCRIPTION_ID`
has **no default** (unlike every sibling `*_SUBSCRIPTION_ID` var), and this
is deliberate: those sibling subscriptions already exist in every deployed
environment, but the diagnostics one does not yet. If it defaulted to a
name, the subscriber would try to open a listener against a subscription
that was never provisioned; since a listener error cancels the subscriber's
root context (`cmd/reearth-flow-subscriber/main.go`), that would crash-loop
the *entire* subscriber process — taking log/node/job ingestion down with
it, not just diagnostics. Leaving it unset keeps `conf.DiagnosticSubscriptionID
!= ""` false until step 2 below explicitly opts an environment in.

Turning this on safely requires bringing the pieces up in a specific order,
because the two failure modes are asymmetric:

- Publishing before the subscription exists **drops messages** — Pub/Sub
  cannot buffer for a subscription that isn't there yet.
- Deploying the subscriber before the engine publishes is **safe**: with
  `REEARTH_FLOW_SUBSCRIBER_DIAGNOSTIC_SUBSCRIPTION_ID` unset (or the
  subscription itself absent), the subscriber simply skips starting the
  diagnostic listener (see `conf.DiagnosticSubscriptionID != ""` in
  `cmd/reearth-flow-subscriber/main.go`) and the rest of the subscriber runs
  unaffected.

Deploy order:

1. **Provision the topic and its pull subscription.**
   - Local (docker compose): already handled by `engine/compose.yml`'s
     pubsub emulator `TOPIC_IDS`, which auto-creates a pull subscription
     named `{topic}-sub` for every listed topic. **Known naming gap:** the
     emulator's `TOPIC_IDS` currently lists `flow-worker-diagnostic-topic`,
     which does **not** match the engine's own default topic name
     (`flow-diagnostic-topic`, above) — unlike the other four topics in that
     list, which all match their Rust defaults exactly. Until that entry is
     corrected, running the worker manually against the local emulator (as
     in "Run the workflow" below) needs
     `FLOW_WORKER_DIAGNOSTIC_TOPIC=flow-worker-diagnostic-topic` exported
     explicitly so publishing lands on the topic the emulator actually
     created; otherwise the worker's default (`flow-diagnostic-topic`)
     publishes to a topic with no subscription and every message is dropped.
   - Production/GCP: provision a topic named `flow-diagnostic-topic`
     (matching the engine default) — or any name, with
     `FLOW_WORKER_DIAGNOSTIC_TOPIC` set to match — plus a pull-type
     subscription for it.
2. **Deploy the subscriber with the new env var set** —
   `REEARTH_FLOW_SUBSCRIBER_DIAGNOSTIC_SUBSCRIPTION_ID` pointed at the
   subscription from step 1. This is safe ahead of step 3: with the
   topic/subscription provisioned but the engine flag still off, the
   subscriber's listener just sits idle waiting for messages that aren't
   sent yet.
3. **Only then flip the engine's `FLOW_WORKER_ENABLE_DIAGNOSTICS=true`.**
   With a live subscription in place and a deployed subscriber pulling from
   it, enabling publishing is safe: no messages are dropped, and every
   `DiagnosticEvent` lands in the subscriber's Redis `diagnostics:*` lists
   and the Mongo `nodeDiagnostics` collection (see the env table above).

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
        name: Feature Creator
        type: action
        action: Feature Creator
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
        name: JSON Writer
        type: action
        action: JSON Writer
        with:
          output: |
            file::join_path(env.get("workerArtifactPath"), env.get("outputPath"))

    edges:
      - id: c064cf52-705f-443a-b2de-6795266c540d
        from: 90f40a3e-61d3-48e2-a328-e7226c2ad1ae
        to: f5e66920-24c0-4c70-ae16-6be1ed3b906c
        fromPort: features
        toPort: features
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
docker exec -it subscriber-redis redis-cli
127.0.0.1:6379> KEYS *
 1) "log:00caad2a-9f7d-4189-b479-153fa9ea36dc:5566c900-9581-4c5c-be02-fd13e4d93669:2025-01-11T09:12:54.943837Z"
 2) "log:00caad2a-9f7d-4189-b479-153fa9ea36dc:5566c900-9581-4c5c-be02-fd13e4d93669:2025-01-11T09:12:54.602634Z"
 3) "log:00caad2a-9f7d-4189-b479-153fa9ea36dc:5566c900-9581-4c5c-be02-fd13e4d93669:2025-01-11T09:12:54.487779Z"
127.0.0.1:6379> get "log:00caad2a-9f7d-4189-b479-153fa9ea36dc:5566c900-9581-4c5c-be02-fd13e4d93669:2025-01-11T09:12:54.487779Z"
```
**Example Output**
```
"{\"workflowId\":\"00caad2a-9f7d-4189-b479-153fa9ea36dc\",\"jobId\":\"5566c900-9581-4c5c-be02-fd13e4d93669\",\"nodeId\":\"f5e66920-24c0-4c70-ae16-6be1ed3b906c\",\"logLevel\":\"INFO\",\"timestamp\":\"2025-01-11T09:12:54.487779Z\",\"message\":\"\\\"JSON Writer\\\" sink start...\"}"
"{\"workflowId\":\"00caad2a-9f7d-4189-b479-153fa9ea36dc\",\"jobId\":\"5566c900-9581-4c5c-be02-fd13e4d93669\",\"nodeId\":\"\",\"logLevel\":\"INFO\",\"timestamp\":\"2025-01-11T09:12:54.602634Z\",\"message\":\"\\\"Feature Creator\\\" finish source complete. elapsed = 855.334\xc2\xb5s\"}"
"{\"workflowId\":\"00caad2a-9f7d-4189-b479-153fa9ea36dc\",\"jobId\":\"5566c900-9581-4c5c-be02-fd13e4d93669\",\"nodeId\":\"f5e66920-24c0-4c70-ae16-6be1ed3b906c\",\"logLevel\":\"INFO\",\"timestamp\":\"2025-01-11T09:12:54.943837Z\",\"message\":\"\\\"JSON Writer\\\" sink finish. elapsed = 1.688292ms\"}"
```


### (Optional) Pulling Logs Dilectly
```
curl -X POST "http://localhost:8085/v1/projects/local-project/subscriptions/flow-log-stream-topic-sub:pull" \
     -H "Content-Type: application/json" \
     -d '{
           "maxMessages": 1000
         }'
```