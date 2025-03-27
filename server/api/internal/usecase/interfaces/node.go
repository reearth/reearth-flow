package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type NodeExecution interface {
	FindByJobNodeID(ctx context.Context, jobID id.JobID, nodeID string) (*graph.NodeExecution, error)
	GetNodeExecutions(ctx context.Context, jobID id.JobID) ([]*graph.NodeExecution, error)
	GetNodeExecution(ctx context.Context, jobID id.JobID, nodeID string) (*graph.NodeExecution, error)
	SubscribeToNode(ctx context.Context, jobID id.JobID, nodeID string) (chan *graph.NodeExecution, error)
	UnsubscribeFromNode(jobID id.JobID, nodeID string, ch chan *graph.NodeExecution)
}
