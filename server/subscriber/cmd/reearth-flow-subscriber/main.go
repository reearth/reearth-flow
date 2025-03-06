package main

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"sync"
	"syscall"
	"time"

	"cloud.google.com/go/pubsub"
	"github.com/redis/go-redis/v9"

	flow_pubsub "github.com/reearth/reearth-flow/subscriber/internal/adapter/pubsub"
	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure"
	flow_redis "github.com/reearth/reearth-flow/subscriber/internal/infrastructure/redis"
	"github.com/reearth/reearth-flow/subscriber/internal/usecase/interactor"
)

func main() {
	ctx, cancel := context.WithCancel(context.Background())
	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)

	conf, cerr := ReadConfig(true)
	if cerr != nil {
		log.Fatalf("failed to load config: %v", cerr)
	}
	log.Printf("config: %s", conf.Print())

	pubsubClient, err := pubsub.NewClient(ctx, conf.GCPProject)
	if err != nil {
		log.Fatalf("Failed to create pubsub client: %v", err)
	}
	defer func() {
		if cerr := pubsubClient.Close(); cerr != nil {
			log.Printf("failed to close pubsub client: %v", cerr)
		}
	}()

	sub := pubsubClient.Subscription(conf.LogSubscriptionID)
	subAdapter := flow_pubsub.NewRealSubscription(sub)

	opt, err := redis.ParseURL(conf.RedisURL)
	if err != nil {
		log.Fatalf("Failed to parse Redis URL: %v", err)
	}

	rdb := redis.NewClient(opt)
	if err := rdb.Ping(ctx).Err(); err != nil {
		log.Fatalf("Failed to connect to Redis: %v", err)
	}

	redisStorage := flow_redis.NewRedisStorage(rdb)
	storageImpl := infrastructure.NewStorageImpl(redisStorage)
	logSubscriberUC := interactor.NewLogSubscriberUseCase(storageImpl)
	subscriber := flow_pubsub.NewSubscriber(subAdapter, logSubscriberUC)

	log.Println("[subscriber] Starting subscriber...")

	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		if err := subscriber.StartListening(ctx); err != nil {
			log.Printf("[subscriber] Subscriber error: %v", err)
			cancel()
		}
	}()

	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		if _, err := fmt.Fprintf(w, "Subscriber is running"); err != nil {
			log.Printf("failed to write response: %v", err)
		}
	})

	http.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		if _, err := fmt.Fprintf(w, "OK"); err != nil {
			log.Printf("failed to write response: %v", err)
		}
	})

	server := &http.Server{
		Addr:    ":" + conf.Port,
		Handler: http.DefaultServeMux,
	}

	go func() {
		log.Printf("[subscriber] Starting HTTP server on port %s...", conf.Port)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Printf("[subscriber] HTTP server error: %v", err)
			cancel()
		}
	}()

	go func() {
		sig := <-sigCh
		log.Printf("[subscriber] Received signal: %v. Shutting down...", sig)

		shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer shutdownCancel()

		if err := server.Shutdown(shutdownCtx); err != nil {
			log.Printf("[subscriber] HTTP server shutdown error: %v", err)
		}

		cancel()
	}()

	wg.Wait()
	log.Println("[subscriber] Subscriber stopped gracefully.")
}
