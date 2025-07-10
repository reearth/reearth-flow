lint:
	# Temporary workaround until golangci-lint supports Go 1.24
	./lint-wrapper.sh

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

gen-policies:
	go run ./cmd/policy-generator

.PHONY: lint test e2e build run-app run-db gql gen-policies all gen-thrift

all: gen-thrift

gen-thrift:
	@echo "Generating Thrift code..."
	@thrift -r --gen go:package_prefix=github.com/reearth/reearth-flow/api/ -out . thrift/document.thrift
