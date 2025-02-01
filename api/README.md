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