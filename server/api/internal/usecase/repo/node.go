package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type NodeExecution interface {
	FindByJobNodeID(ctx context.Context, jobID id.JobID, nodeID string) (*graph.NodeExecution, error)
}
