package infrastructure

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/subscriber/pkg/job"
)

type JobStorageImpl struct {
	live LiveStore
}

func NewJobStorageImpl(live LiveStore) gateway.JobStorage {
	return &JobStorageImpl{
		live: live,
	}
}

func (s *JobStorageImpl) Save(ctx context.Context, event *job.JobCompleteEvent) error {
	return s.live.SaveJobComplete(ctx, event)
}
