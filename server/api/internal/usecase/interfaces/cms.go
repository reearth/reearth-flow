package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/cms"
)

type CMS interface {
	GetCMSProject(ctx context.Context, projectIDOrAlias string) (*cms.Project, error)

	ListCMSProjects(ctx context.Context, workspaceIDs []string, publicOnly bool, page, pageSize *int32) (*cms.ListProjectsOutput, error)

	GetCMSAsset(ctx context.Context, assetID string) (*cms.Asset, error)

	ListCMSAssets(ctx context.Context, projectID string, page, pageSize *int32) (*cms.ListAssetsOutput, error)

	GetCMSModel(ctx context.Context, projectIDOrAlias, modelIDOrAlias string) (*cms.Model, error)

	ListCMSModels(ctx context.Context, projectID string, page, pageSize *int32) (*cms.ListModelsOutput, error)

	ListCMSItems(ctx context.Context, projectID, modelID string, keyword *string, page, pageSize *int32) (*cms.ListItemsOutput, error)

	GetCMSModelExportURL(ctx context.Context, projectID, modelID string) (string, error)
}
