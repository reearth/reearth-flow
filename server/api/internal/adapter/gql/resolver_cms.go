package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
)

func (r *queryResolver) CmsProject(ctx context.Context, projectIDOrAlias gqlmodel.ID) (*gqlmodel.CMSProject, error) {
	return nil, nil
}

func (r *queryResolver) CmsProjects(ctx context.Context, workspaceID gqlmodel.ID, publicOnly *bool) ([]*gqlmodel.CMSProject, error) {
	return nil, nil
}

func (r *queryResolver) CmsModels(ctx context.Context, projectID gqlmodel.ID) ([]*gqlmodel.CMSModel, error) {
	return nil, nil
}

func (r *queryResolver) CmsItems(ctx context.Context, projectID gqlmodel.ID, modelID gqlmodel.ID, page *int, pageSize *int) (*gqlmodel.CMSItemsConnection, error) {
	return nil, nil
}

func (r *queryResolver) CmsModelExportURL(ctx context.Context, projectID gqlmodel.ID, modelID gqlmodel.ID) (string, error) {
	return "", nil
}
