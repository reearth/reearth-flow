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
	log.Printf("INFO: Starting reearth-flow subscriber service")

	ctx, cancel := context.WithCancel(context.Background())
	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)
	log.Printf("DEBUG: Context and signal handlers initialized")

	conf, cerr := ReadConfig(true)
	if cerr != nil {
		log.Fatalf("FATAL: Failed to load config: %v", cerr)
	}
	log.Printf("INFO: Configuration loaded successfully: %s", conf.Print())

	log.Printf("DEBUG: Initializing clients")
	pubsubClient, redisClient, mongoClient := initClients(ctx, conf)
	defer cleanupClients(pubsubClient, redisClient, mongoClient)
	log.Printf("INFO: All clients initialized successfully")

	log.Printf("DEBUG: Initializing storage containers")
	storages := initStorages(ctx, conf, redisClient, mongoClient)
	log.Printf("INFO: Storage containers initialized successfully")

	log.Printf("DEBUG: Initializing subscribers")
	subscribers := initSubscribers(ctx, conf, pubsubClient, storages)
	log.Printf("INFO: Subscribers initialized successfully")

	log.Printf("DEBUG: Starting HTTP server")
	server := startHTTPServer(conf.Port)
	log.Printf("INFO: HTTP server started on port %s", conf.Port)

	var wg sync.WaitGroup
	log.Printf("DEBUG: Starting subscriber listeners")
	startSubscribers(ctx, &wg, subscribers, cancel)
	log.Printf("INFO: All subscriber listeners started")

	log.Printf("DEBUG: Setting up graceful shutdown handler")
	handleGracefulShutdown(ctx, sigCh, server, cancel)
	log.Printf("INFO: Graceful shutdown handler configured")

	log.Printf("INFO: Service is now fully operational")
	wg.Wait()
	log.Println("INFO: All subscribers stopped gracefully")
}

func initClients(ctx context.Context, conf *Config) (*pubsub.Client, *redis.Client, *mongo.Client) {
	log.Printf("DEBUG: Creating PubSub client for project %s", conf.GCPProject)
	pubsubClient, err := pubsub.NewClient(ctx, conf.GCPProject)
	if err != nil {
		log.Fatalf("FATAL: Failed to create PubSub client: %v", err)
	}
	log.Printf("DEBUG: PubSub client created successfully")

	log.Printf("DEBUG: Parsing Redis URL %s", maskSensitiveURL(conf.RedisURL))
	redisOpt, err := redis.ParseURL(conf.RedisURL)
	if err != nil {
		log.Fatalf("FATAL: Failed to parse Redis URL: %v", err)
	}

	log.Printf("DEBUG: Creating Redis client")
	redisClient := redis.NewClient(redisOpt)

	log.Printf("DEBUG: Testing Redis connection")
	if err := redisClient.Ping(ctx).Err(); err != nil {
		log.Fatalf("FATAL: Failed to connect to Redis: %v", err)
	}
	log.Printf("DEBUG: Redis connection successful")

	log.Printf("DEBUG: Creating MongoDB client for %s", maskSensitiveURL(conf.DB))
	mongoClient, err := mongo.Connect(ctx, options.Client().ApplyURI(conf.DB).SetMonitor(otelmongo.NewMonitor()))
	if err != nil {
		log.Fatalf("FATAL: Failed to connect to MongoDB: %v", err)
	}

	log.Printf("DEBUG: Testing MongoDB connection")
	if err := mongoClient.Ping(ctx, nil); err != nil {
		log.Fatalf("FATAL: Failed to ping MongoDB: %v", err)
	}
	log.Printf("DEBUG: MongoDB connection successful")

	return pubsubClient, redisClient, mongoClient
}

// Mask sensitive parts of URLs for logging
func maskSensitiveURL(url string) string {
	// This is a simple implementation - in production you might want something more robust
	// that properly handles different URL formats
	return url // For this example, we're not implementing the masking
}

func cleanupClients(pubsubClient *pubsub.Client, redisClient *redis.Client, mongoClient *mongo.Client) {
	log.Printf("DEBUG: Starting cleanup of clients")

	if pubsubClient != nil {
		log.Printf("DEBUG: Closing PubSub client")
		if err := pubsubClient.Close(); err != nil {
			log.Printf("ERROR: Failed to close PubSub client: %v", err)
		} else {
			log.Printf("DEBUG: PubSub client closed successfully")
		}
	}

	if redisClient != nil {
		log.Printf("DEBUG: Closing Redis client")
		if err := redisClient.Close(); err != nil {
			log.Printf("ERROR: Failed to close Redis client: %v", err)
		} else {
			log.Printf("DEBUG: Redis client closed successfully")
		}
	}

	if mongoClient != nil {
		log.Printf("DEBUG: Disconnecting MongoDB client")
		if err := mongoClient.Disconnect(context.Background()); err != nil {
			log.Printf("ERROR: Failed to close MongoDB client: %v", err)
		} else {
			log.Printf("DEBUG: MongoDB client disconnected successfully")
		}
	}

	log.Printf("DEBUG: Client cleanup completed")
}

type StorageContainers struct {
	Redis *flow_redis.RedisStorage
	Mongo *flow_mongo.MongoStorage
	Log   gateway.LogStorage
	Edge  gateway.EdgeStorage
}

func initStorages(ctx context.Context, conf *Config, redisClient *redis.Client, mongoClient *mongo.Client) *StorageContainers {
	log.Printf("DEBUG: Initializing Redis storage")
	redisStorage := flow_redis.NewRedisStorage(redisClient)
	log.Printf("DEBUG: Redis storage initialized")

	log.Printf("DEBUG: Initializing MongoDB storage with bucket=%s, baseURL=%s", conf.GCSBucket, conf.AssetBaseURL)
	mongoStorage := flow_mongo.NewMongoStorage(
		mongox.NewClient(databaseName, mongoClient),
		conf.GCSBucket,
		conf.AssetBaseURL,
	)
	log.Printf("DEBUG: MongoDB storage initialized")

	log.Printf("DEBUG: Initializing Log storage implementation")
	logStorage := infrastructure.NewLogStorageImpl(redisStorage)
	log.Printf("DEBUG: Log storage implementation initialized")

	log.Printf("DEBUG: Initializing Edge storage implementation")
	edgeStorage := infrastructure.NewEdgeStorageImpl(redisStorage, mongoStorage)
	log.Printf("DEBUG: Edge storage implementation initialized")

	return &StorageContainers{
		Redis: redisStorage,
		Mongo: mongoStorage,
		Log:   logStorage,
		Edge:  edgeStorage,
	}
}

type Subscribers struct {
	Log  *flow_pubsub.LogSubscriber
	Edge *flow_pubsub.EdgeSubscriber
}

func initSubscribers(_ context.Context, conf *Config, pubsubClient *pubsub.Client, storages *StorageContainers) *Subscribers {
	var logSubscriber *flow_pubsub.LogSubscriber
	if conf.LogSubscriptionID != "" {
		log.Printf("DEBUG: Initializing Log subscriber with subscription ID: %s", conf.LogSubscriptionID)
		logSub := pubsubClient.Subscription(conf.LogSubscriptionID)
		logSubAdapter := flow_pubsub.NewRealSubscription(logSub)
		logSubscriberUC := interactor.NewLogSubscriberUseCase(storages.Log)
		logSubscriber = flow_pubsub.NewLogSubscriber(logSubAdapter, logSubscriberUC)
		log.Printf("DEBUG: Log subscriber initialized")
	} else {
		log.Printf("WARN: Log subscription ID not provided, Log subscriber will not be initialized")
	}

	var edgeSubscriber *flow_pubsub.EdgeSubscriber
	if conf.EdgeSubscriptionID != "" {
		log.Printf("DEBUG: Initializing Edge subscriber with subscription ID: %s", conf.EdgeSubscriptionID)
		edgeSub := pubsubClient.Subscription(conf.EdgeSubscriptionID)
		edgeSubAdapter := flow_pubsub.NewRealSubscription(edgeSub)
		edgeSubscriberUC := interactor.NewEdgeSubscriberUseCase(storages.Edge)
		edgeSubscriber = flow_pubsub.NewEdgeSubscriber(edgeSubAdapter, edgeSubscriberUC)
		log.Printf("DEBUG: Edge subscriber initialized")
	} else {
		log.Printf("WARN: Edge subscription ID not provided, Edge subscriber will not be initialized")
	}

	return &Subscribers{
		Log:  logSubscriber,
		Edge: edgeSubscriber,
	}
}

func startHTTPServer(port string) *http.Server {
	log.Printf("DEBUG: Setting up HTTP endpoints")

	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		log.Printf("DEBUG: Received request at / from %s", r.RemoteAddr)
		if _, err := fmt.Fprintf(w, "Subscriber service is running"); err != nil {
			log.Printf("ERROR: Failed to write response for / endpoint: %v", err)
		}
	})

	http.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		log.Printf("DEBUG: Received health check request from %s", r.RemoteAddr)
		w.WriteHeader(http.StatusOK)
		if _, err := fmt.Fprintf(w, "OK"); err != nil {
			log.Printf("ERROR: Failed to write response for /health endpoint: %v", err)
		}
	})

	server := &http.Server{
		Addr:    ":" + port,
		Handler: http.DefaultServeMux,
	}

	go func() {
		log.Printf("INFO: Starting HTTP server on port %s", port)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Printf("ERROR: HTTP server error: %v", err)
		}
	}()

	return server
}

func startSubscribers(ctx context.Context, wg *sync.WaitGroup, subscribers *Subscribers, cancel context.CancelFunc) {
	if subscribers.Log != nil {
		wg.Add(1)
		go func() {
			defer wg.Done()
			log.Printf("INFO: Starting Log subscriber...")
			if err := subscribers.Log.StartListening(ctx); err != nil {
				log.Printf("ERROR: Log subscriber error: %v", err)
				log.Printf("INFO: Cancelling context due to Log subscriber error")
				cancel()
			}
			log.Printf("DEBUG: Log subscriber.StartListening returned")
		}()
	} else {
		log.Printf("WARN: Log subscriber not configured, skipping...")
	}

	if subscribers.Edge != nil {
		wg.Add(1)
		go func() {
			defer wg.Done()
			log.Printf("INFO: Starting Edge subscriber...")
			if err := subscribers.Edge.StartListening(ctx); err != nil {
				log.Printf("ERROR: Edge subscriber error: %v", err)
				log.Printf("INFO: Cancelling context due to Edge subscriber error")
				cancel()
			}
			log.Printf("DEBUG: Edge subscriber.StartListening returned")
		}()
	} else {
		log.Printf("WARN: Edge subscriber not configured, skipping...")
	}
}

func handleGracefulShutdown(_ context.Context, sigCh chan os.Signal, server *http.Server, cancel context.CancelFunc) {
	go func() {
		sig := <-sigCh
		log.Printf("INFO: Received signal: %v. Starting graceful shutdown...", sig)

		shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer shutdownCancel()

		log.Printf("DEBUG: Shutting down HTTP server")
		if err := server.Shutdown(shutdownCtx); err != nil {
			log.Printf("ERROR: HTTP server shutdown error: %v", err)
		} else {
			log.Printf("DEBUG: HTTP server shutdown successful")
		}

		log.Printf("DEBUG: Canceling main context")
		cancel()
		log.Printf("DEBUG: Main context canceled")
	}()
}
