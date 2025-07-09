package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/cms"
)

type CMS interface {
	GetCMSProject(ctx context.Context, projectIDOrAlias string) (*cms.Project, error)

	ListCMSProjects(ctx context.Context, workspaceID string, publicOnly bool) ([]*cms.Project, int32, error)

	ListCMSModels(ctx context.Context, projectID string) ([]*cms.Model, int32, error)

	ListCMSItems(ctx context.Context, projectID, modelID string, page, pageSize *int32) (*cms.ListItemsOutput, error)

	GetCMSModelExportURL(ctx context.Context, projectID, modelID string) (string, error)
}
