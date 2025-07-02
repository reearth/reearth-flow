package gateway

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/cms"
)

// CMS is the gateway interface for interacting with ReEarth CMS
type CMS interface {
	// Project operations
	GetProject(ctx context.Context, projectIDOrAlias string) (*cms.Project, error)
	ListProjects(ctx context.Context, input cms.ListProjectsInput) ([]*cms.Project, int32, error)

	// Model operations
	ListModels(ctx context.Context, input cms.ListModelsInput) ([]*cms.Model, int32, error)

	// Item operations
	ListItems(ctx context.Context, input cms.ListItemsInput) (*cms.ListItemsOutput, error)

	// Export operations
	GetModelGeoJSONExportURL(ctx context.Context, input cms.ExportInput) (*cms.ExportOutput, error)
}
