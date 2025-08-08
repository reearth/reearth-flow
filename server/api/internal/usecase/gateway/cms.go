package gateway

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/cms"
)

type CMS interface {
	GetProject(ctx context.Context, projectIDOrAlias string) (*cms.Project, error)
	ListProjects(ctx context.Context, input cms.ListProjectsInput) (*cms.ListProjectsOutput, error)

	GetAsset(ctx context.Context, input cms.GetAssetInput) (*cms.Asset, error)
	ListAssets(ctx context.Context, input cms.ListAssetsInput) (*cms.ListAssetsOutput, error)

	GetModel(ctx context.Context, input cms.GetModelInput) (*cms.Model, error)
	ListModels(ctx context.Context, input cms.ListModelsInput) (*cms.ListModelsOutput, error)

	ListItems(ctx context.Context, input cms.ListItemsInput) (*cms.ListItemsOutput, error)

	GetModelGeoJSONExportURL(ctx context.Context, input cms.ExportInput) (*cms.ExportOutput, error)
}
