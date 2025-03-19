package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/edge"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type EdgeExecution interface {
	FindByJobEdgeID(ctx context.Context, jobID id.JobID, edgeID string) (*edge.EdgeExecution, error)
	GetEdgeExecutions(ctx context.Context, jobID id.JobID) ([]*edge.EdgeExecution, error)
	GetEdgeExecution(ctx context.Context, jobID id.JobID, edgeID string) (*edge.EdgeExecution, error)
	SubscribeToEdge(ctx context.Context, jobID id.JobID, edgeID string) (chan *edge.EdgeExecution, error)
	UnsubscribeFromEdge(jobID id.JobID, edgeID string, ch chan *edge.EdgeExecution)
}
