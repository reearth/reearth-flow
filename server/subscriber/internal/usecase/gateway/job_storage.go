package gateway

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/pkg/job"
)

type JobStorage interface {
	Save(ctx context.Context, event *job.JobCompleteEvent) error
}
