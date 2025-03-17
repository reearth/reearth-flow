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

	pubsubClient, redisClient, mongoClient := initClients(ctx, conf)
	defer cleanupClients(pubsubClient, redisClient, mongoClient)

	storages := initStorages(ctx, conf, redisClient, mongoClient)

	subscribers := initSubscribers(ctx, conf, pubsubClient, storages)

	server := startHTTPServer(conf.Port)

	var wg sync.WaitGroup
	startSubscribers(ctx, &wg, subscribers, cancel)

	handleGracefulShutdown(ctx, sigCh, server, cancel)

	wg.Wait()
	log.Println("[subscriber] All subscribers stopped gracefully.")
}

func initClients(ctx context.Context, conf *Config) (*pubsub.Client, *redis.Client, *mongo.Client) {
	pubsubClient, err := pubsub.NewClient(ctx, conf.GCPProject)
	if err != nil {
		log.Fatalf("Failed to create pubsub client: %v", err)
	}

	redisOpt, err := redis.ParseURL(conf.RedisURL)
	if err != nil {
		log.Fatalf("Failed to parse Redis URL: %v", err)
	}
	redisClient := redis.NewClient(redisOpt)
	if err := redisClient.Ping(ctx).Err(); err != nil {
		log.Fatalf("Failed to connect to Redis: %v", err)
	}

	mongoClient, err := mongo.Connect(ctx, options.Client().ApplyURI(conf.DB).SetMonitor(otelmongo.NewMonitor()))
	if err != nil {
		log.Fatalf("Failed to connect to MongoDB: %v", err)
	}

	if err := mongoClient.Ping(ctx, nil); err != nil {
		log.Fatalf("Failed to ping MongoDB: %v", err)
	}

	return pubsubClient, redisClient, mongoClient
}

func cleanupClients(pubsubClient *pubsub.Client, redisClient *redis.Client, mongoClient *mongo.Client) {
	if pubsubClient != nil {
		if err := pubsubClient.Close(); err != nil {
			log.Printf("failed to close pubsub client: %v", err)
		}
	}

	if redisClient != nil {
		if err := redisClient.Close(); err != nil {
			log.Printf("failed to close redis client: %v", err)
		}
	}

	if mongoClient != nil {
		if err := mongoClient.Disconnect(context.Background()); err != nil {
			log.Printf("failed to close mongodb client: %v", err)
		}
	}
}

type StorageContainers struct {
	Redis *flow_redis.RedisStorage
	Mongo *flow_mongo.MongoStorage
	Log   gateway.LogStorage
	Edge  gateway.EdgeStorage
}

func initStorages(ctx context.Context, conf *Config, redisClient *redis.Client, mongoClient *mongo.Client) *StorageContainers {
	redisStorage := flow_redis.NewRedisStorage(redisClient)

	mongoStorage := flow_mongo.NewMongoStorage(
		mongox.NewClient(databaseName, mongoClient),
		conf.GCSBucket,
		conf.AssetBaseURL,
	)

	logStorage := infrastructure.NewLogStorageImpl(redisStorage)
	edgeStorage := infrastructure.NewEdgeStorageImpl(redisStorage, mongoStorage)

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
		logSub := pubsubClient.Subscription(conf.LogSubscriptionID)
		logSubAdapter := flow_pubsub.NewRealSubscription(logSub)
		logSubscriberUC := interactor.NewLogSubscriberUseCase(storages.Log)
		logSubscriber = flow_pubsub.NewLogSubscriber(logSubAdapter, logSubscriberUC)
	}

	var edgeSubscriber *flow_pubsub.EdgeSubscriber
	if conf.EdgeSubscriptionID != "" {
		edgeSub := pubsubClient.Subscription(conf.EdgeSubscriptionID)
		edgeSubAdapter := flow_pubsub.NewRealSubscription(edgeSub)
		edgeSubscriberUC := interactor.NewEdgeSubscriberUseCase(storages.Edge)
		edgeSubscriber = flow_pubsub.NewEdgeSubscriber(edgeSubAdapter, edgeSubscriberUC)
	}

	return &Subscribers{
		Log:  logSubscriber,
		Edge: edgeSubscriber,
	}
}

func startHTTPServer(port string) *http.Server {
	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		if _, err := fmt.Fprintf(w, "Subscriber service is running"); err != nil {
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
		Addr:    ":" + port,
		Handler: http.DefaultServeMux,
	}

	go func() {
		log.Printf("[subscriber] Starting HTTP server on port %s...", port)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Printf("[subscriber] HTTP server error: %v", err)
		}
	}()

	return server
}

func startSubscribers(ctx context.Context, wg *sync.WaitGroup, subscribers *Subscribers, cancel context.CancelFunc) {
	wg.Add(1)
	go func() {
		defer wg.Done()
		log.Println("[subscriber] Starting log subscriber...")
		if err := subscribers.Log.StartListening(ctx); err != nil {
			log.Printf("[subscriber] Log subscriber error: %v", err)
			cancel()
		}
	}()

	if subscribers.Edge != nil {
		wg.Add(1)
		go func() {
			defer wg.Done()
			log.Println("[subscriber] Starting edge subscriber...")
			if err := subscribers.Edge.StartListening(ctx); err != nil {
				log.Printf("[subscriber] Edge subscriber error: %v", err)
				cancel()
			}
		}()
	} else {
		log.Println("[subscriber] Edge subscriber not configured, skipping...")
	}
}

func handleGracefulShutdown(_ context.Context, sigCh chan os.Signal, server *http.Server, cancel context.CancelFunc) {
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
}
