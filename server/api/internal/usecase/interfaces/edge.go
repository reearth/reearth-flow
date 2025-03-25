package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type EdgeExecution interface {
	FindByJobEdgeID(ctx context.Context, jobID id.JobID, edgeID string) (*graph.EdgeExecution, error)
}
