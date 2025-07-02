package gateway

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/cms"
)

// CMS is the gateway interface for interacting with ReEarth CMS
type CMS interface {
	GetProject(ctx context.Context, projectIDOrAlias string) (*cms.Project, error)
	ListProjects(ctx context.Context, input cms.ListProjectsInput) ([]*cms.Project, int32, error)

	CreateProject(ctx context.Context, input cms.CreateProjectInput) (*cms.Project, error)
	UpdateProject(ctx context.Context, input cms.UpdateProjectInput) (*cms.Project, error)
	DeleteProject(ctx context.Context, input cms.DeleteProjectInput) (*cms.DeleteProjectOutput, error)
	CheckAliasAvailability(ctx context.Context, input cms.CheckAliasAvailabilityInput) (*cms.CheckAliasAvailabilityOutput, error)

	ListModels(ctx context.Context, input cms.ListModelsInput) ([]*cms.Model, int32, error)

	ListItems(ctx context.Context, input cms.ListItemsInput) (*cms.ListItemsOutput, error)

	GetModelGeoJSONExportURL(ctx context.Context, input cms.ExportInput) (*cms.ExportOutput, error)
}
