package gateway

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/pkg/edge"
)

type EdgeStorage interface {
	SaveToRedis(ctx context.Context, event *edge.PassThroughEvent) error
	UpdateEdgeStatusInMongo(ctx context.Context, jobID string, edge *edge.EdgeExecution) error
	ConstructIntermediateDataURL(jobID, edgeID string) string
}
