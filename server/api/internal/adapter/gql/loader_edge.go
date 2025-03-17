package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type EdgeExLoader struct {
	usecase interfaces.EdgeExecution
}

func NewEdgeExLoader(usecase interfaces.EdgeExecution) *EdgeExLoader {
	return &EdgeExLoader{usecase: usecase}
}

func (c *EdgeExLoader) FindByJobEdgeID(ctx context.Context, jobID gqlmodel.ID, edgeId string) (*gqlmodel.EdgeExecution, error) {
	jId, err := id.JobIDFrom(string(jobID))
	if err != nil {
		return nil, err
	}

	edgeEx, err := c.usecase.FindByJobEdgeID(ctx, jId, edgeId)
	if err != nil {
		return nil, err
	}

	return gqlmodel.ToEdgeExecution(edgeEx), nil
}
