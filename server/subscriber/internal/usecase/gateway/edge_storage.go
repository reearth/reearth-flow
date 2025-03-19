package gateway

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/pkg/edge"
)

type EdgeStorage interface {
	ConstructIntermediateDataURL(jobID, edgeID string) string
	FindEdgeExecution(ctx context.Context, jobID string, edgeID string) (*edge.EdgeExecution, error)
	SaveToRedis(ctx context.Context, event *edge.PassThroughEvent) error
	UpdateEdgeStatusInMongo(ctx context.Context, jobID string, edge *edge.EdgeExecution) error
}
