package gateway

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/pkg/node"
)

type NodeStorage interface {
	SaveToRedis(ctx context.Context, event *node.NodeStatusEvent) error
	SaveToMongo(ctx context.Context, jobID string, nodeExecution *node.NodeExecution) error
}
