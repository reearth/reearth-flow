package gateway

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/pkg/job"
)

type JobStorage interface {
	SaveToRedis(ctx context.Context, event *job.JobStatusEvent) error
	SaveToMongo(ctx context.Context, jobID string, jobRecord *job.Job) error
}

type APIGateway interface {
	NotifyJobStatusChange(ctx context.Context, jobID string, status string) error
}
