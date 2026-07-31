test:
	go test -race -short -v ./...

run:
	docker-compose up --build -d

lint:
	golangci-lint run --fix

sqlc:
	cd internal/infrastructure/postgres && sqlc generate
