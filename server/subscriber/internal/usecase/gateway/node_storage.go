package gateway

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/pkg/node"
)

type NodeStorage interface {
	FindNodeExecution(ctx context.Context, jobID string, edgeID string) (*node.NodeExecution, error)
	SaveToRedis(ctx context.Context, event *node.NodeStatusEvent) error
	SaveToMongo(ctx context.Context, jobID string, nodeExecution *node.NodeExecution) error
}
