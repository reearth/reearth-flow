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
	"github.com/reearth/reearthx/mongox"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
	"go.opentelemetry.io/contrib/instrumentation/go.mongodb.org/mongo-driver/mongo/otelmongo"

	flow_pubsub "github.com/reearth/reearth-flow/subscriber/internal/adapter/pubsub"
	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure"
	flow_mongo "github.com/reearth/reearth-flow/subscriber/internal/infrastructure/mongo"
	flow_redis "github.com/reearth/reearth-flow/subscriber/internal/infrastructure/redis"
	"github.com/reearth/reearth-flow/subscriber/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/subscriber/internal/usecase/interactor"
)

const databaseName = "reearth-flow"

func main() {
	ctx, cancel := context.WithCancel(context.Background())
	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)

	conf, cerr := ReadConfig(true)
	if cerr != nil {
		log.Fatalf("failed to load config: %v", cerr)
	}
	log.Printf("config: %s", conf.Print())

	// Initialize PubSub client
	pubsubClient, err := pubsub.NewClient(ctx, conf.GCPProject)
	if err != nil {
		log.Fatalf("Failed to create pubsub client: %v", err)
	}
	defer func() {
		if cerr := pubsubClient.Close(); cerr != nil {
			log.Printf("failed to close pubsub client: %v", cerr)
		}
	}()

	// Initialize Redis client
	opt, err := redis.ParseURL(conf.RedisURL)
	if err != nil {
		log.Fatalf("Failed to parse Redis URL: %v", err)
	}
	redisClient := redis.NewClient(opt)
	if err := redisClient.Ping(ctx).Err(); err != nil {
		log.Fatalf("Failed to connect to Redis: %v", err)
	}
	defer func() {
		if rerr := redisClient.Close(); rerr != nil {
			log.Printf("failed to close redis client: %v", rerr)
		}
	}()

	// Initialize storage components
	redisStorage := flow_redis.NewRedisStorage(redisClient)
	logStorage := infrastructure.NewLogStorageImpl(redisStorage)

	// Initialize MongoDB client and node storage if needed
	var mongoClient *mongo.Client
	var nodeStorage gateway.NodeStorage

	if conf.NodeSubscriptionID != "" {
		mongoClient, err = mongo.Connect(ctx, options.Client().ApplyURI(conf.DB).SetMonitor(otelmongo.NewMonitor()))
		if err != nil {
			log.Fatalf("Failed to connect to MongoDB: %v", err)
		}
		if err := mongoClient.Ping(ctx, nil); err != nil {
			log.Fatalf("Failed to ping MongoDB: %v", err)
		}

		defer func() {
			if merr := mongoClient.Disconnect(context.Background()); merr != nil {
				log.Printf("failed to disconnet mongo client: %v", merr)
			}
		}()

		mongoStorage := flow_mongo.NewMongoStorage(
			mongox.NewClient(databaseName, mongoClient),
			conf.GCSBucket,
			conf.AssetBaseURL,
		)
		nodeStorage = infrastructure.NewNodeStorageImpl(redisStorage, mongoStorage)
	}

	// Set up subscribers with respective subscriptions
	var wg sync.WaitGroup

	// Set up log subscriber if configured
	if conf.LogSubscriptionID != "" {
		logSub := pubsubClient.Subscription(conf.LogSubscriptionID)
		logSubAdapter := flow_pubsub.NewRealSubscription(logSub)
		logSubscriberUC := interactor.NewLogSubscriberUseCase(logStorage)
		logSubscriber := flow_pubsub.NewLogSubscriber(logSubAdapter, logSubscriberUC)

		wg.Add(1)
		go func() {
			defer wg.Done()
			log.Println("[subscriber] Starting log subscriber...")
			if err := logSubscriber.StartListening(ctx); err != nil {
				log.Printf("[subscriber] Log subscriber error: %v", err)
				cancel()
			}
			log.Println("[subscriber] Log subscriber stopped")
		}()
	} else {
		log.Println("Log subscription ID not provided, log subscriber will not be started")
	}

	// Set up worker stdout log subscriber if configured
	if conf.WorkerStdoutLogSubscriptionID != "" {
		workerStdoutLogSub := pubsubClient.Subscription(conf.WorkerStdoutLogSubscriptionID)
		workerStdoutLogSubAdapter := flow_pubsub.NewRealSubscription(workerStdoutLogSub)
		logSubscriberUC := interactor.NewLogSubscriberUseCase(logStorage)
		workerStdoutLogSubscriber := flow_pubsub.NewLogSubscriber(workerStdoutLogSubAdapter, logSubscriberUC)

		wg.Add(1)
		go func() {
			defer wg.Done()
			log.Println("[subscriber] Starting worker stdout log subscriber...")
			if err := workerStdoutLogSubscriber.StartListening(ctx); err != nil {
				log.Printf("[subscriber] Worker stdout log subscriber error: %v", err)
				cancel()
			}
			log.Println("[subscriber] Worker stdout log subscriber stopped")
		}()
	} else {
		log.Println("Worker stdout log subscription ID not provided, worker stdout log subscriber will not be started")
	}

	// Set up node subscriber if configured
	if conf.NodeSubscriptionID != "" && nodeStorage != nil {
		nodeSub := pubsubClient.Subscription(conf.NodeSubscriptionID)
		nodeSubAdapter := flow_pubsub.NewRealSubscription(nodeSub)
		nodeSubscriberUC := interactor.NewNodeSubscriberUseCase(nodeStorage)
		nodeSubscriber := flow_pubsub.NewNodeSubscriber(nodeSubAdapter, nodeSubscriberUC)

		wg.Add(1)
		go func() {
			defer wg.Done()
			log.Println("[subscriber] Starting node subscriber...")
			if err := nodeSubscriber.StartListening(ctx); err != nil {
				log.Printf("[subscriber] Node subscriber error: %v", err)
				cancel()
			}
			log.Println("[subscriber] Node subscriber stopped")
		}()
	} else if conf.NodeSubscriptionID != "" {
		log.Println("Node storage not properly initialized, node subscriber will not be started")
	} else {
		log.Println("Node subscription ID not provided, node subscriber will not be started")
	}

	// Set up HTTP server
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

	// Set up graceful shutdown handler
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
	log.Println("[subscriber] All subscribers stopped gracefully.")
}
