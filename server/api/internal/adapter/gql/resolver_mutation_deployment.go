package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
)

func (r *mutationResolver) CreateDeployment(ctx context.Context, input gqlmodel.CreateDeploymentInput) (*gqlmodel.DeploymentPayload, error) {
	var pid *id.ProjectID
	if input.ProjectID != nil {
		p, err := gqlmodel.ToID[id.Project](*input.ProjectID)
		if err != nil {
			return nil, err
		}
		pid = &p
	}

	wsid, err := gqlmodel.ToID[accountdomain.Workspace](input.WorkspaceID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Deployment.Create(ctx, interfaces.CreateDeploymentParam{
		Project:     pid,
		Workspace:   wsid,
		Workflow:    gqlmodel.FromFile(&input.File),
		Description: input.Description,
	})
	if err != nil {
		return nil, err
	}
	return &gqlmodel.DeploymentPayload{Deployment: gqlmodel.ToDeployment(res)}, nil
}

func (r *mutationResolver) UpdateDeployment(ctx context.Context, input gqlmodel.UpdateDeploymentInput) (*gqlmodel.DeploymentPayload, error) {
	did, err := gqlmodel.ToID[id.Deployment](input.DeploymentID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Deployment.Update(ctx, interfaces.UpdateDeploymentParam{
		ID:          did,
		Workflow:    gqlmodel.FromFile(input.File),
		Description: input.Description,
	})
	if err != nil {
		return nil, err
	}

	return &gqlmodel.DeploymentPayload{Deployment: gqlmodel.ToDeployment(res)}, nil
}

func (r *mutationResolver) DeleteDeployment(ctx context.Context, input gqlmodel.DeleteDeploymentInput) (*gqlmodel.DeleteDeploymentPayload, error) {
	did, err := gqlmodel.ToID[id.Deployment](input.DeploymentID)
	if err != nil {
		return nil, err
	}

	if err := usecases(ctx).Deployment.Delete(ctx, did); err != nil {
		return nil, err
	}

	return &gqlmodel.DeleteDeploymentPayload{DeploymentID: input.DeploymentID}, nil
}

func (r *mutationResolver) ExecuteDeployment(ctx context.Context, input gqlmodel.ExecuteDeploymentInput) (*gqlmodel.JobPayload, error) {
	did, err := gqlmodel.ToID[id.Deployment](input.DeploymentID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Deployment.Execute(ctx, interfaces.ExecuteDeploymentParam{
		DeploymentID: did,
	})
	if err != nil {
		return nil, err
	}

	return &gqlmodel.JobPayload{Job: gqlmodel.ToJob(res)}, nil
}
