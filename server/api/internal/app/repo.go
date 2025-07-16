package app

import (
	"context"
	"fmt"
	"strconv"

	"github.com/redis/go-redis/v9"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/asyncq"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/auth0"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/fs"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gcpbatch"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gcpscheduler"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gcs"
	mongorepo "github.com/reearth/reearth-flow/api/internal/infrastructure/mongo"
	redisrepo "github.com/reearth/reearth-flow/api/internal/infrastructure/redis"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interactor"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearthx/account/accountinfrastructure/accountmongo"
	"github.com/reearth/reearthx/account/accountusecase/accountgateway"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/mongox"
	"github.com/spf13/afero"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
	"go.opentelemetry.io/contrib/instrumentation/go.mongodb.org/mongo-driver/mongo/otelmongo"
)

const (
	databaseName        = "reearth-flow"
	accountDatabaseName = "reearth-account"
)

func initReposAndGateways(ctx context.Context, conf *config.Config, _ bool) (*repo.Container, *gateway.Container, *accountrepo.Container, *accountgateway.Container) {
	interactor.InitWebsocket(
		conf.WebsocketThriftServerURL,
	)

	gateways := &gateway.Container{}
	acGateways := &accountgateway.Container{}

	// Mongo
	client, err := mongo.Connect(
		ctx,
		options.Client().
			ApplyURI(conf.DB).
			SetMonitor(otelmongo.NewMonitor()),
	)
	if err != nil {
		log.Fatalf("mongo error: %+v\n", err)
	}

	accountDatabase := conf.DB_Account
	accountRepoCompat := false
	if accountDatabase == "" {
		accountDatabase = accountDatabaseName
		accountRepoCompat = true
	}

	accountUsers := make([]accountrepo.User, 0, len(conf.DB_Users))
	for _, u := range conf.DB_Users {
		c, err := mongo.Connect(ctx, options.Client().ApplyURI(u.URI).SetMonitor(otelmongo.NewMonitor()))
		if err != nil {
			log.Fatalf("mongo error: %+v\n", err)
		}
		accountUsers = append(accountUsers, accountmongo.NewUserWithHost(mongox.NewClient(accountDatabase, c), u.Name))
	}

	txAvailable := mongox.IsTransactionAvailable(conf.DB)

	accountRepos, err := accountmongo.New(ctx, client, accountDatabase, txAvailable, accountRepoCompat, accountUsers)
	if err != nil {
		log.Fatalf("Failed to init mongo: %+v\n", err)
	}

	repos, err := mongorepo.New(ctx, client.Database(databaseName), accountRepos, txAvailable)
	if err != nil {
		log.Fatalf("Failed to init mongo: %+v\n", err)
	}

	gateways.Redis = initRedis(ctx, conf)

	gateways.File = initFile(ctx, conf)

	gateways.Batch = initBatch(ctx, conf)

	gateways.Scheduler = initScheduler(ctx, conf)

	auth0 := auth0.New(conf.Auth0.Domain, conf.Auth0.ClientID, conf.Auth0.ClientSecret)
	gateways.Authenticator = auth0
	acGateways.Authenticator = auth0

	return repos, gateways, accountRepos, acGateways
}

func initFile(ctx context.Context, conf *config.Config) (fileRepo gateway.File) {
	var err error
	if conf.GCS.IsConfigured() {
		log.Infofc(ctx, "file: GCS storage is used: %s\n", conf.GCS.BucketName)
		fileRepo, err = gcs.NewFile(conf.GCS.BucketName, conf.AssetBaseURL, conf.GCS.PublicationCacheControl)
		if err != nil {
			log.Warnf("file: failed to init GCS storage: %s\n", err.Error())
		}
		return
	}

	log.Infof("file: local storage is used")
	afs := afero.NewBasePathFs(afero.NewOsFs(), "data")
	fileRepo, err = fs.NewFile(afs, conf.AssetBaseURL, conf.WorkflowBaseURL)
	if err != nil {
		log.Fatalf("file: init error: %+v", err)
	}
	return fileRepo
}

func initBatch(ctx context.Context, conf *config.Config) (batchRepo gateway.Batch) {
	if conf.Worker_ImageURL == "" {
		return nil
	}

	log.Infof("Initializing Asyncq + GCP Batch hybrid mode")

	_, err := initGCPBatch(ctx, conf)
	if err != nil {
		log.Fatalf("Failed to initialize GCP Batch for hybrid mode: %v", err)
	}

	asyncqConfig := &asyncq.Config{
		RedisAddr:     conf.Worker_AsyncqRedisAddr,
		RedisPassword: conf.Worker_AsyncqRedisPassword,
		RedisDB:       conf.Worker_AsyncqRedisDB,
		Concurrency:   conf.Worker_AsyncqConcurrency,
		MaxRetry:      conf.Worker_AsyncqMaxRetry,
		Queues: map[string]int{
			"critical": 6,
			"default":  3,
			"low":      1,
		},
	}

	asyncqBatch, err := asyncq.NewAsyncqBatch(asyncqConfig)
	if err != nil {
		log.Fatalf("Failed to initialize Asyncq batch: %v", err)
	}

	log.Infof("Hybrid mode initialized: Asyncq queue with GCP Batch execution")
	return asyncqBatch
}

func initGCPBatch(ctx context.Context, conf *config.Config) (gateway.Batch, error) {
	if conf.GCPProject == "" {
		return nil, fmt.Errorf("GCP project ID is required")
	}
	if conf.GCPRegion == "" {
		return nil, fmt.Errorf("GCP region is required")
	}

	bootDiskSize, err := strconv.Atoi(conf.Worker_BootDiskSizeGB)
	if err != nil {
		return nil, fmt.Errorf("invalid boot disk size: %v", err)
	}

	computeCpuMilli, err := strconv.Atoi(conf.Worker_ComputeCpuMilli)
	if err != nil {
		return nil, fmt.Errorf("invalid compute CPU milli: %v", err)
	}

	computeMemoryMib, err := strconv.Atoi(conf.Worker_ComputeMemoryMib)
	if err != nil {
		return nil, fmt.Errorf("invalid compute memory MiB: %v", err)
	}

	taskCount, err := strconv.Atoi(conf.Worker_TaskCount)
	if err != nil {
		return nil, fmt.Errorf("invalid task count: %v", err)
	}

	batchConfig := gcpbatch.BatchConfig{
		AllowedLocations:                conf.Worker_AllowedLocations,
		BinaryPath:                      conf.Worker_BinaryPath,
		BootDiskSizeGB:                  bootDiskSize,
		BootDiskType:                    conf.Worker_BootDiskType,
		ComputeCpuMilli:                 computeCpuMilli,
		ComputeMemoryMib:                computeMemoryMib,
		ImageURI:                        conf.Worker_ImageURL,
		MachineType:                     conf.Worker_MachineType,
		NodeStatusPropagationDelayMS:    conf.Worker_NodeStatusPropagationDelayMS,
		PubSubEdgePassThroughEventTopic: conf.Worker_PubSubEdgePassThroughEventTopic,
		PubSubLogStreamTopic:            conf.Worker_PubSubLogStreamTopic,
		PubSubJobCompleteTopic:          conf.Worker_PubSubJobCompleteTopic,
		PubSubNodeStatusTopic:           conf.Worker_PubSubNodeStatusTopic,
		ProjectID:                       conf.GCPProject,
		Region:                          conf.GCPRegion,
		SAEmail:                         conf.Worker_BatchSAEmail,
		TaskCount:                       taskCount,
	}

	return gcpbatch.NewBatch(ctx, batchConfig)
}

func initRedis(ctx context.Context, conf *config.Config) gateway.Redis {
	if conf.Redis_URL == "" {
		return nil
	}

	log.Infofc(ctx, "log: redis storage is used: %s\n", conf.Redis_URL)
	opt, err := redis.ParseURL(conf.Redis_URL)
	if err != nil {
		log.Fatalf("failed to parse redis url: %s\n", err.Error())
	}
	client := redis.NewClient(opt)
	RedisRepo, err := redisrepo.NewRedisLog(client)
	if err != nil {
		log.Warnf("log: failed to init redis storage: %s\n", err.Error())
	}
	return RedisRepo
}

func initScheduler(ctx context.Context, conf *config.Config) gateway.Scheduler {
	if conf.GCPProject == "" || conf.GCPRegion == "" {
		log.Info("Scheduler disabled: GCP project or region not configured")
		return nil
	}

	config := gcpscheduler.SchedulerConfig{
		ProjectID: conf.GCPProject,
		Location:  conf.GCPRegion,
		Host:      conf.Host,
	}

	scheduler, err := gcpscheduler.NewScheduler(ctx, config)
	if err != nil {
		log.Errorf("failed to create Scheduler: %v", err)
		return nil
	}

	log.Infofc(ctx, "Scheduler enabled for project %s in region %s targeting %s", conf.GCPProject, conf.GCPRegion, conf.Host)
	return scheduler
}
