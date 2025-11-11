package gateway

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/pkg/job"
)

type JobStorage interface {
	SaveToRedis(ctx context.Context, event *job.JobCompleteEvent) error
}
