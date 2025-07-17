package gateway

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/cms"
)

type CMS interface {
	GetProject(ctx context.Context, projectIDOrAlias string) (*cms.Project, error)
	ListProjects(ctx context.Context, input cms.ListProjectsInput) ([]*cms.Project, int32, error)

	ListModels(ctx context.Context, input cms.ListModelsInput) ([]*cms.Model, int32, error)

	ListItems(ctx context.Context, input cms.ListItemsInput) (*cms.ListItemsOutput, error)

	GetModelGeoJSONExportURL(ctx context.Context, input cms.ExportInput) (*cms.ExportOutput, error)
}
