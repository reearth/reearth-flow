package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
)

func (r *mutationResolver) CreateDeployment(ctx context.Context, input gqlmodel.CreateDeploymentInput) (*gqlmodel.DeploymentPayload, error) {
	pid, err := gqlmodel.ToID[id.Project](input.ProjectID)
	if err != nil {
		return nil, err
	}

	wsid, err := gqlmodel.ToID[accountdomain.Workspace](input.WorkspaceID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Deployment.Create(ctx, interfaces.CreateDeploymentParam{
		Project:   pid,
		Workspace: wsid,
		Workflow:  gqlmodel.FromFile(&input.File),
	}, getOperator(ctx))
	if err != nil {
		return nil, err
	}

	return &gqlmodel.DeploymentPayload{Deployment: gqlmodel.ToDeployment(res)}, nil
}

func (r *mutationResolver) ExecuteDeployment(ctx context.Context, input gqlmodel.ExecuteDeploymentInput) (*gqlmodel.JobPayload, error) {
	did, err := gqlmodel.ToID[id.Deployment](input.DeploymentID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Deployment.Execute(ctx, interfaces.ExecuteDeploymentParam{
		DeploymentID: did,
	}, getOperator(ctx))
	if err != nil {
		return nil, err
	}

	return &gqlmodel.JobPayload{Job: gqlmodel.ToJob(res)}, nil
}
