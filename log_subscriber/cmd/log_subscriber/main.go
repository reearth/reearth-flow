package main

import (
	"context"
	"log"
	"os"
	"os/signal"
	"sync"
	"syscall"

	"cloud.google.com/go/pubsub"
	"github.com/redis/go-redis/v9"

	flow_pubsub "github.com/reearth/reearth-flow/log-subscriber/internal/adapter/pubsub"
	"github.com/reearth/reearth-flow/log-subscriber/internal/infrastructure"
	flow_redis "github.com/reearth/reearth-flow/log-subscriber/internal/infrastructure/redis"
	"github.com/reearth/reearth-flow/log-subscriber/internal/usecase/interactor"
)

func getEnv(key, defaultVal string) string {
	val := os.Getenv(key)
	if val == "" {
		return defaultVal
	}
	return val
}

func main() {
	ctx, cancel := context.WithCancel(context.Background())
	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)

	projectID := getEnv("FLOW_LOG_SUBSCRIBER_PROJECT_ID", "local-project")
	subscriptionID := getEnv("FLOW_LOG_SUBSCRIBER_SUBSCRIPTION_ID", "flow-log-stream-topic-sub")
	pubsubEmulatorHost := getEnv("PUBSUB_EMULATOR_HOST", "")
	if pubsubEmulatorHost != "" {
		log.Printf("Using Pub/Sub emulator: %s\n", pubsubEmulatorHost)
	}

	pubsubClient, err := pubsub.NewClient(ctx, projectID)
	if err != nil {
		log.Fatalf("Failed to create pubsub client: %v", err)
	}
	defer func() {
		if cerr := pubsubClient.Close(); cerr != nil {
			log.Printf("failed to close pubsub client: %v", cerr)
		}
	}()

	sub := pubsubClient.Subscription(subscriptionID)
	subAdapter := flow_pubsub.NewRealSubscription(sub)

	redisAddr := getEnv("FLOW_LOG_SUBSCRIBER_REDIS_ADDR", "localhost:6379")
	redisPassword := getEnv("FLOW_LOG_SUBSCRIBER_REDIS_PASSWORD", "")
	rdb := redis.NewClient(&redis.Options{
		Addr:     redisAddr,
		Password: redisPassword,
		DB:       0,
	})
	if err := rdb.Ping(ctx).Err(); err != nil {
		log.Fatalf("Failed to connect to Redis: %v", err)
	}

	redisStorage := flow_redis.NewRedisStorage(rdb)

	storageImpl := infrastructure.NewStorageImpl(redisStorage)

	logSubscriberUC := interactor.NewLogSubscriberUseCase(storageImpl)

	subscriber := flow_pubsub.NewSubscriber(subAdapter, logSubscriberUC)

	log.Println("[log_subscriber] Starting subscriber...")

	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		if err := subscriber.StartListening(ctx); err != nil {
			log.Printf("[log_subscriber] Subscriber error: %v", err)
			cancel()
		}
	}()

	go func() {
		sig := <-sigCh
		log.Printf("[log_subscriber] Received signal: %v. Shutting down...", sig)
		cancel()
	}()

	wg.Wait()
	log.Println("[log_subscriber] Subscriber stopped gracefully.")
}
