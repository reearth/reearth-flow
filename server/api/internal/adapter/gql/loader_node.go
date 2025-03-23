package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type NodeExLoader struct {
	usecase interfaces.NodeExecution
}

func NewNodeExLoader(usecase interfaces.NodeExecution) *NodeExLoader {
	return &NodeExLoader{usecase: usecase}
}

func (c *NodeExLoader) FindByJobNodeID(ctx context.Context, jobID gqlmodel.ID, nodeId string) (*gqlmodel.NodeExecution, error) {
	jId, err := id.JobIDFrom(string(jobID))
	if err != nil {
		return nil, err
	}

	nodeEx, err := c.usecase.FindByJobNodeID(ctx, jId, nodeId)
	if err != nil {
		return nil, err
	}

	return gqlmodel.ToNodeExecution(nodeEx), nil
}
