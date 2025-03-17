package gateway

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/edge"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
)

type Redis interface {
	GetLogs(ctx context.Context, since time.Time, until time.Time, jobID id.JobID) ([]*log.Log, error)
	GetEdgeExecutions(ctx context.Context, jobID id.JobID) ([]*edge.EdgeExecution, error)
	GetEdgeExecution(ctx context.Context, jobID id.JobID, edgeID string) (*edge.EdgeExecution, error)
}
