package app

import (
	"context"
	"strconv"

	"github.com/redis/go-redis/v9"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/auth0"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/cms"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/fs"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gcpbatch"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gcpscheduler"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gcs"
	mongorepo "github.com/reearth/reearth-flow/api/internal/infrastructure/mongo"
	redisrepo "github.com/reearth/reearth-flow/api/internal/infrastructure/redis"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
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

	accountDatabase := conf.DbAccount
	accountRepoCompat := false
	if accountDatabase == "" {
		accountDatabase = accountDatabaseName
		accountRepoCompat = true
	}

	accountUsers := make([]accountrepo.User, 0, len(conf.DbUsers))
	for _, u := range conf.DbUsers {
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
	// Redis
	gateways.Redis = initRedis(ctx, conf)

	// File
	gateways.File = initFile(ctx, conf)

	// Batch
	gateways.Batch = initBatch(ctx, conf)

	// Scheduler
	gateways.Scheduler = initScheduler(ctx, conf)

	// Auth0
	auth0 := auth0.New(conf.Auth0.Domain, conf.Auth0.ClientID, conf.Auth0.ClientSecret)
	acGateways.Authenticator = auth0

	// CMS
	gateways.CMS = initCMS(ctx, conf)

	return repos, gateways, accountRepos, acGateways
}

func initFile(ctx context.Context, conf *config.Config) (fileRepo gateway.File) {
	var err error
	if conf.GCS.IsConfigured() {
		log.Infofc(ctx, "file: GCS storage is used: %s\n", conf.GCS.BucketName)
		fileRepo, err = gcs.NewFile(conf.GCS.BucketName, conf.AssetBaseURL, conf.GCS.PublicationCacheControl, conf.AssetUploadURLReplacement)
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
	if conf.WorkerImageurl == "" {
		return nil
	}

	if conf.GCPProject == "" {
		log.Fatal("GCP project ID is required")
	}
	if conf.GCPRegion == "" {
		log.Fatal("GCP region is required")
	}

	bootDiskSize, err := strconv.Atoi(conf.WorkerBootdisksizegb)
	if err != nil {
		log.Fatalf("invalid boot disk size: %v", err)
	}

	computeCpuMilli, err := strconv.Atoi(conf.WorkerComputecpumilli)
	if err != nil {
		log.Fatalf("invalid boot disk size: %v", err)
	}

	computeMemoryMib, err := strconv.Atoi(conf.WorkerComputememorymib)
	if err != nil {
		log.Fatalf("invalid task count: %v", err)
	}

	taskCount, err := strconv.Atoi(conf.WorkerTaskcount)
	if err != nil {
		log.Fatalf("invalid task count: %v", err)
	}

	config := gcpbatch.BatchConfig{
		AllowedLocations:                conf.WorkerAllowedlocations,
		BinaryPath:                      conf.WorkerBinarypath,
		BootDiskSizeGB:                  bootDiskSize,
		BootDiskType:                    conf.WorkerBootdisktype,
		ChannelBufferSize:               conf.WorkerChannelbuffersize,
		ComputeCpuMilli:                 computeCpuMilli,
		ComputeMemoryMib:                computeMemoryMib,
		FeatureFlushThreshold:           conf.WorkerFeatureflushthreshold,
		ImageURI:                        conf.WorkerImageurl,
		MachineType:                     conf.WorkerMachinetype,
		NodeStatusPropagationDelayMS:    conf.WorkerNodestatuspropagationdelayms,
		PubSubEdgePassThroughEventTopic: conf.WorkerPubsubedgepassthrougheventtopic,
		PubSubLogStreamTopic:            conf.WorkerPubsublogstreamtopic,
		PubSubJobCompleteTopic:          conf.WorkerPubsubjobcompletetopic,
		PubSubNodeStatusTopic:           conf.WorkerPubsubnodestatustopic,
		PubSubUserFacingLogTopic:        conf.WorkerPubsubuserfacinglogtopic,
		ProjectID:                       conf.GCPProject,
		Region:                          conf.GCPRegion,
		RustLog:                         conf.WorkerRustlog,
		SAEmail:                         conf.WorkerBatchsaemail,
		TaskCount:                       taskCount,
		ThreadPoolSize:                  conf.WorkerThreadpoolsize,
		CompressIntermediateData:        conf.WorkerCompressintermediatedata,
	}

	batchRepo, err = gcpbatch.NewBatch(ctx, config)
	if err != nil {
		log.Fatalf("failed to create Batch repository: %v", err)
	}

	return
}

func initRedis(ctx context.Context, conf *config.Config) gateway.Redis {
	if conf.RedisUrl == "" {
		return nil
	}

	log.Infofc(ctx, "log: redis storage is used: %s\n", conf.RedisUrl)
	opt, err := redis.ParseURL(conf.RedisUrl)
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

func initCMS(ctx context.Context, conf *config.Config) gateway.CMS {
	if conf.CmsEndpoint == "" {
		log.Info("CMS disabled: endpoint not configured")
		return nil
	}

	if conf.CmsToken == "" {
		log.Warn("CMS: no authentication token provided")
	}

	cmsClient, err := cms.NewGRPCClient(conf.CmsEndpoint, conf.CmsToken, conf.CmsUsetls)
	if err != nil {
		log.Errorf("failed to create CMS client: %v", err)
		return nil
	}

	log.Infofc(ctx, "CMS enabled: endpoint=%s", conf.CmsEndpoint)
	return cmsClient
}
