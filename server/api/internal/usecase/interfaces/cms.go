package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/cms"
)

// CMS defines the usecase interface for CMS operations in Flow
type CMS interface {
	// GetCMSProject retrieves a CMS project by ID or alias
	GetCMSProject(ctx context.Context, projectIDOrAlias string) (*cms.Project, error)

	// ListCMSProjects lists CMS projects for a workspace
	ListCMSProjects(ctx context.Context, workspaceID string, publicOnly bool) ([]*cms.Project, int32, error)

	// ListCMSModels lists models for a CMS project
	ListCMSModels(ctx context.Context, projectID string) ([]*cms.Model, int32, error)

	// ListCMSItems lists items for a CMS model
	ListCMSItems(ctx context.Context, projectID, modelID string, page, pageSize *int32) (*cms.ListItemsOutput, error)

	// GetCMSModelExportURL gets the GeoJSON export URL for a CMS model
	GetCMSModelExportURL(ctx context.Context, projectID, modelID string) (string, error)
}
