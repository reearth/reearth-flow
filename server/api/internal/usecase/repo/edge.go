package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/edge"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type EdgeExecution interface {
	FindByJobEdgeID(ctx context.Context, jobID id.JobID, edgeID string) (*edge.EdgeExecution, error)
}
