package gql

import (
	"context"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

func (r *mutationResolver) PreviewSchema(ctx context.Context, input gqlmodel.PreviewSchemaInput) (*gqlmodel.PreviewSchemaPayload, error) {
	pid, err := gqlmodel.ToID[id.Project](input.ProjectID)
	if err != nil {
		return nil, err
	}

	if _, err := gqlmodel.ToID[accountsid.Workspace](input.WorkspaceID); err != nil {
		return nil, err
	}

	parameters, err := gqlmodel.FromRunParameters(input.Parameters)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Project.PreviewSchema(ctx, interfaces.PreviewSchemaParam{
		ProjectID:  pid,
		Workflow:   gqlmodel.FromFile(&input.File),
		Parameters: parameters,
		SampleSize: input.SampleSize,
	})
	if err != nil {
		return nil, err
	}

	return &gqlmodel.PreviewSchemaPayload{Job: gqlmodel.ToJob(res)}, nil
}
