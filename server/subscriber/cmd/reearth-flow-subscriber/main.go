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
	flow_mongo "github.com/reearth/reearth-flow/subscriber/internal/infrastructure/mongo"
	flow_redis "github.com/reearth/reearth-flow/subscriber/internal/infrastructure/redis"
	"github.com/reearth/reearth-flow/subscriber/internal/usecase/interactor"
)

type Subscriber interface {
	StartListening(ctx context.Context) error
}

type clients struct {
	pubsub *pubsub.Client
	redis  *redis.Client
	mongo  flow_mongo.MongoClient
}

func main() {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	
	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)

	conf, err := ReadConfig(true)
	if err != nil {
		log.Fatalf("failed to load config: %v", err)
	}
	log.Printf("config: %s", conf.Print())

	clients, err := initClients(ctx, conf)
	if err != nil {
		log.Fatalf("Failed to initialize clients: %v", err)
	}
	defer closeClients(ctx, clients)

	subscribers, err := initSubscribers(ctx, conf, clients)
	if err != nil {
		log.Fatalf("Failed to initialize subscribers: %v", err)
	}

	server := startHTTPServer(conf.Port)

	var wg sync.WaitGroup
	runSubscribers(ctx, &wg, subscribers)

	go handleShutdown(sigCh, server, cancel)

	wg.Wait()
	log.Println("[subscriber] Subscribers stopped gracefully.")
}

func initClients(ctx context.Context, conf *Config) (*clients, error) {
	pubsubClient, err := pubsub.NewClient(ctx, conf.GCPProject)
	if err != nil {
		return nil, fmt.Errorf("failed to create pubsub client: %w", err)
	}

	redisOpt, err := redis.ParseURL(conf.RedisURL)
	if err != nil {
		return nil, fmt.Errorf("failed to parse Redis URL: %w", err)
	}
	redisClient := redis.NewClient(redisOpt)
	if err := redisClient.Ping(ctx).Err(); err != nil {
		return nil, fmt.Errorf("failed to connect to Redis: %w", err)
	}

	mongoClient, err := flow_mongo.NewMongoClient(ctx, conf.MongoURI, conf.MongoDatabaseName)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to MongoDB: %w", err)
	}

	return &clients{
		pubsub: pubsubClient,
		redis:  redisClient,
		mongo:  mongoClient,
	}, nil
}

func closeClients(ctx context.Context, c *clients) {
	if c.pubsub != nil {
		if err := c.pubsub.Close(); err != nil {
			log.Printf("failed to close pubsub client: %v", err)
		}
	}
	
	if c.redis != nil {
		if err := c.redis.Close(); err != nil {
			log.Printf("failed to close Redis client: %v", err)
		}
	}
	
	if c.mongo != nil {
		if err := c.mongo.Disconnect(ctx); err != nil {
			log.Printf("failed to disconnect MongoDB client: %v", err)
		}
	}
}

func initSubscribers(ctx context.Context, conf *Config, c *clients) ([]Subscriber, error) {
	redisStorage := flow_redis.NewRedisStorage(c.redis)
	mongoStorage := flow_mongo.NewMongoStorage(
		c.mongo,
		conf.MongoJobCollection,
		"graphs",
		"edges",
	)

	logStorageImpl := infrastructure.NewStorageImpl(redisStorage)
	logSubscriberUC := interactor.NewLogSubscriberUseCase(logStorageImpl)
	
	logSub := c.pubsub.Subscription(conf.LogSubscriptionID)
	logSubAdapter := flow_pubsub.NewRealSubscription(logSub)
	logSubscriber := flow_pubsub.NewSubscriber(logSubAdapter, logSubscriberUC)

	nodeStatusStorageImpl := infrastructure.NewNodeStatusStorageImpl(redisStorage, mongoStorage)
	nodeStatusSubscriberUC := interactor.NewNodeStatusSubscriberUseCase(nodeStatusStorageImpl)
	
	edgePassSub := c.pubsub.Subscription(conf.EdgePassSubscriptionID)
	edgePassSubAdapter := flow_pubsub.NewRealSubscription(edgePassSub)
	edgePassSubscriber := flow_pubsub.NewEdgeSubscriber(edgePassSubAdapter, nodeStatusSubscriberUC)

	return []Subscriber{logSubscriber, edgePassSubscriber}, nil
}

func startHTTPServer(port string) *http.Server {
	mux := http.NewServeMux()
	
	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		if _, err := fmt.Fprintf(w, "Subscriber is running"); err != nil {
			log.Printf("failed to write response: %v", err)
		}
	})

	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		if _, err := fmt.Fprintf(w, "OK"); err != nil {
			log.Printf("failed to write response: %v", err)
		}
	})

	server := &http.Server{
		Addr:         ":" + port,
		Handler:      mux,
		ReadTimeout:  15 * time.Second,
		WriteTimeout: 15 * time.Second,
		IdleTimeout:  60 * time.Second,
	}

	go func() {
		log.Printf("[subscriber] Starting HTTP server on port %s...", port)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Printf("[subscriber] HTTP server error: %v", err)
		}
	}()
	
	return server
}

func runSubscribers(ctx context.Context, wg *sync.WaitGroup, subscribers []Subscriber) {
	log.Println("[subscriber] Starting subscribers...")
	
	for i, sub := range subscribers {
		wg.Add(1)
		
		go func(index int, s Subscriber) {
			defer wg.Done()
			subscriberName := fmt.Sprintf("Subscriber-%d", index+1)
			
			log.Printf("[%s] Starting...", subscriberName)
			if err := s.StartListening(ctx); err != nil {
				log.Printf("[%s] Error: %v", subscriberName, err)
			}
			log.Printf("[%s] Stopped", subscriberName)
		}(i, sub)
	}
}

func handleShutdown(sigCh chan os.Signal, server *http.Server, cancel context.CancelFunc) {
	sig := <-sigCh
	log.Printf("[subscriber] Received signal: %v. Shutting down...", sig)

	shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer shutdownCancel()

	if err := server.Shutdown(shutdownCtx); err != nil {
		log.Printf("[subscriber] HTTP server shutdown error: %v", err)
	}

	cancel()
}
