package main

import (
	"context"
	"os"
	"os/signal"
	"syscall"

	"github.com/redis/go-redis/v9"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/asyncq"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/fs"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gcs"
	mongorepo "github.com/reearth/reearth-flow/api/internal/infrastructure/mongo"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearthx/account/accountinfrastructure/accountmongo"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/mongox"
	"github.com/spf13/afero"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
	"go.opentelemetry.io/contrib/instrumentation/go.mongodb.org/mongo-driver/mongo/otelmongo"
)

func main() {
	log.Info("Starting asyncq worker...")

	ctx := context.Background()

	conf, err := config.ReadConfig(false)
	if err != nil {
		log.Fatalf("failed to load config: %v", err)
	}

	repos, fileGateway, err := initRepos(ctx, conf)
	if err != nil {
		log.Fatalf("failed to initialize repositories: %v", err)
	}

	asyncqConfig := getAsyncqConfig(conf)

	batchGateway, err := asyncq.NewAsyncqBatch(asyncqConfig)
	if err != nil {
		log.Fatalf("failed to create batch gateway: %v", err)
	}

	worker := asyncq.NewAsyncqWorker(
		asyncqConfig,
		repos.Job,
		fileGateway,
		batchGateway,
	)

	go func() {
		if err := worker.Start(); err != nil {
			log.Fatalf("failed to start worker: %v", err)
		}
	}()

	log.Info("Asyncq worker started successfully")

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, os.Interrupt, syscall.SIGTERM)
	<-quit

	log.Info("Shutting down worker...")
	worker.Stop()
	log.Info("Worker stopped")
}

func initRepos(ctx context.Context, conf *config.Config) (*repo.Container, gateway.File, error) {
	client, err := mongo.Connect(
		ctx,
		options.Client().
			ApplyURI(conf.DB).
			SetMonitor(otelmongo.NewMonitor()),
	)
	if err != nil {
		return nil, nil, err
	}

	accountDatabase := conf.DB_Account
	if accountDatabase == "" {
		accountDatabase = "reearth-account"
	}

	accountUsers := make([]accountrepo.User, 0)
	txAvailable := mongox.IsTransactionAvailable(conf.DB)

	accountRepos, err := accountmongo.New(ctx, client, accountDatabase, txAvailable, false, accountUsers)
	if err != nil {
		return nil, nil, err
	}

	repos, err := mongorepo.New(ctx, client.Database("reearth-flow"), accountRepos, txAvailable)
	if err != nil {
		return nil, nil, err
	}

	var fileGateway gateway.File
	if conf.GCS.IsConfigured() {
		fileGateway, err = gcs.NewFile(conf.GCS.BucketName, conf.AssetBaseURL, conf.GCS.PublicationCacheControl)
		if err != nil {
			return nil, nil, err
		}
	} else {
		afs := afero.NewBasePathFs(afero.NewOsFs(), "data")
		fileGateway, err = fs.NewFile(afs, conf.AssetBaseURL, conf.WorkflowBaseURL)
		if err != nil {
			return nil, nil, err
		}
	}

	return repos, fileGateway, nil
}

func getAsyncqConfig(conf *config.Config) *asyncq.Config {
	redisURL := conf.Batch_Redis_URL
	if redisURL == "" {
		redisURL = conf.Redis_URL
	}

	redisAddr := "localhost:6379"
	redisPassword := conf.Batch_Redis_Password
	redisDB := conf.Batch_Redis_DB

	if redisURL != "" {
		opt, err := redis.ParseURL(redisURL)
		if err != nil {
			log.Fatalf("failed to parse Redis URL: %v", err)
		}
		redisAddr = opt.Addr
		redisPassword = opt.Password
		redisDB = opt.DB
	}

	return &asyncq.Config{
		RedisAddr:     redisAddr,
		RedisPassword: redisPassword,
		RedisDB:       redisDB,
		Concurrency:   conf.Batch_Concurrency,
		MaxRetry:      conf.Batch_MaxRetry,
		Queues: map[string]int{
			"critical": conf.Batch_Queue_Critical,
			"default":  conf.Batch_Queue_Default,
			"low":      conf.Batch_Queue_Low,
		},
	}
}
