package gateway

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
)

type Redis interface {
	GetLogs(ctx context.Context, since time.Time, until time.Time, jobID id.JobID) ([]*log.Log, error)
	GetNodeExecutions(ctx context.Context, jobID id.JobID) ([]*graph.NodeExecution, error)
	GetNodeExecution(ctx context.Context, jobID id.JobID, nodeID string) (*graph.NodeExecution, error)
}
