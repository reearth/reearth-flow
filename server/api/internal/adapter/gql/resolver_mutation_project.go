package gql

import (
	"context"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

func (r *mutationResolver) CreateProject(ctx context.Context, input gqlmodel.CreateProjectInput) (*gqlmodel.ProjectPayload, error) {
	tid, err := gqlmodel.ToID[accountsid.Workspace](input.WorkspaceID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Project.Create(ctx, interfaces.CreateProjectParam{
		Archived:    input.Archived,
		Description: input.Description,
		Name:        input.Name,
		WorkspaceID: tid,
	})
	if err != nil {
		return nil, err
	}

	return &gqlmodel.ProjectPayload{Project: gqlmodel.ToProject(res)}, nil
}

func (r *mutationResolver) UpdateProject(ctx context.Context, input gqlmodel.UpdateProjectInput) (*gqlmodel.ProjectPayload, error) {
	pid, err := gqlmodel.ToID[id.Project](input.ProjectID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Project.Update(ctx, interfaces.UpdateProjectParam{
		Archived:          input.Archived,
		BasicAuthPassword: input.BasicAuthPassword,
		BasicAuthUsername: input.BasicAuthUsername,
		Description:       input.Description,
		ID:                pid,
		IsBasicAuthActive: input.IsBasicAuthActive,
		Name:              input.Name,
	})
	if err != nil {
		return nil, err
	}

	return &gqlmodel.ProjectPayload{Project: gqlmodel.ToProject(res)}, nil
}

func (r *mutationResolver) DeleteProject(ctx context.Context, input gqlmodel.DeleteProjectInput) (*gqlmodel.DeleteProjectPayload, error) {
	pid, err := gqlmodel.ToID[id.Project](input.ProjectID)
	if err != nil {
		return nil, err
	}

	if err := usecases(ctx).Project.Delete(ctx, pid); err != nil {
		return nil, err
	}

	return &gqlmodel.DeleteProjectPayload{ProjectID: input.ProjectID}, nil
}

func (r *mutationResolver) RunProject(ctx context.Context, input gqlmodel.RunProjectInput) (*gqlmodel.RunProjectPayload, error) {
	pid, err := gqlmodel.ToID[id.Project](input.ProjectID)
	if err != nil {
		return nil, err
	}

	_, err = gqlmodel.ToID[accountsid.Workspace](input.WorkspaceID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Project.Run(ctx, interfaces.RunProjectParam{
		ProjectID: pid,
		Workflow:  gqlmodel.FromFile(&input.File),
	})
	if err != nil {
		return nil, err
	}

	return &gqlmodel.RunProjectPayload{Job: gqlmodel.ToJob(res)}, nil
}
