package infrastructure

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/mongo"
	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/redis"
	"github.com/reearth/reearth-flow/subscriber/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
	"github.com/reearth/reearth-flow/subscriber/pkg/job"
	domainLog "github.com/reearth/reearth-flow/subscriber/pkg/log"
	"github.com/reearth/reearth-flow/subscriber/pkg/userfacinglog"
)

// LiveStore is where the job live data the API reads goes: logs, user-facing
// logs, diagnostics and job-complete events. Redis by default, or Postgres when
// REEARTH_FLOW_SUBSCRIBER_LIVE_BACKEND=postgres.
type LiveStore interface {
	SaveLog(ctx context.Context, event *domainLog.LogEvent) error
	SaveUserFacingLog(ctx context.Context, event *userfacinglog.UserFacingLogEvent) error
	SaveDiagnostic(ctx context.Context, event *diagnostic.DiagnosticEvent) error
	SaveJobComplete(ctx context.Context, event *job.JobCompleteEvent) error
}

// redisLiveStore adapts RedisStorage, whose methods predate the LiveStore names.
type redisLiveStore struct {
	r *redis.RedisStorage
}

func NewRedisLiveStore(r *redis.RedisStorage) LiveStore {
	return &redisLiveStore{r: r}
}

func (s *redisLiveStore) SaveLog(ctx context.Context, event *domainLog.LogEvent) error {
	return s.r.SaveLogToRedis(ctx, event)
}

func (s *redisLiveStore) SaveUserFacingLog(ctx context.Context, event *userfacinglog.UserFacingLogEvent) error {
	return s.r.SaveUserFacingLogToRedis(ctx, event)
}

func (s *redisLiveStore) SaveDiagnostic(ctx context.Context, event *diagnostic.DiagnosticEvent) error {
	return s.r.SaveDiagnosticToRedis(ctx, event)
}

func (s *redisLiveStore) SaveJobComplete(ctx context.Context, event *job.JobCompleteEvent) error {
	return s.r.SaveToRedis(ctx, event)
}

type logStorageImpl struct {
	live LiveStore
}

func NewLogStorageImpl(live LiveStore) gateway.LogStorage {
	return &logStorageImpl{
		live: live,
	}
}

func (s *logStorageImpl) Save(ctx context.Context, event *domainLog.LogEvent) error {
	return s.live.SaveLog(ctx, event)
}

type userFacingLogStorageImpl struct {
	live LiveStore
}

func NewUserFacingLogStorageImpl(live LiveStore) gateway.UserFacingLogStorage {
	return &userFacingLogStorageImpl{
		live: live,
	}
}

func (s *userFacingLogStorageImpl) Save(ctx context.Context, event *userfacinglog.UserFacingLogEvent) error {
	return s.live.SaveUserFacingLog(ctx, event)
}

type diagnosticStorageImpl struct {
	live  LiveStore
	mongo *mongo.MongoStorage
}

func NewDiagnosticStorageImpl(live LiveStore, m *mongo.MongoStorage) gateway.DiagnosticStorage {
	return &diagnosticStorageImpl{
		live:  live,
		mongo: m,
	}
}

func (s *diagnosticStorageImpl) Save(ctx context.Context, event *diagnostic.DiagnosticEvent) error {
	return s.live.SaveDiagnostic(ctx, event)
}

func (s *diagnosticStorageImpl) SaveToMongo(ctx context.Context, event *diagnostic.DiagnosticEvent) error {
	return s.mongo.SaveDiagnosticToMongo(ctx, event)
}
