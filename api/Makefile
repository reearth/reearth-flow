lint:
	golangci-lint run --fix

test:
	go test -race -short -v ./...

e2e:
	go test -v ./e2e/...

build:
	go build ./cmd/reearth-flow

run-app:
	go run ./cmd/reearth-flow

run-db:
	docker compose -f ./docker-compose.yml up -d reearth-flow-mongo

gql:
	go generate ./internal/adapter/gql

.PHONY: lint test e2e build run-app run-db gql
