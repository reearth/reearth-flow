package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/cms"
)

// CMS is the Flow-facing CMS usecase. Every project-scoped call takes the
// workspace that owns the CMS project: the CMS internal API only resolves a
// project inside a workspace, and the same id is what Flow checks permission on.
type CMS interface {
	GetCMSProject(ctx context.Context, workspaceID, projectIDOrAlias string) (*cms.Project, error)

	ListCMSProjects(ctx context.Context, workspaceIDs []string, keyword *string, publicOnly bool, page, pageSize *int32) (*cms.ListProjectsOutput, error)

	GetCMSAsset(ctx context.Context, workspaceID, assetID string) (*cms.Asset, error)

	ListCMSAssets(ctx context.Context, workspaceID, projectID string, page, pageSize *int32) (*cms.ListAssetsOutput, error)

	GetCMSModel(ctx context.Context, workspaceID, projectIDOrAlias, modelIDOrAlias string) (*cms.Model, error)

	ListCMSModels(ctx context.Context, workspaceID, projectID string, page, pageSize *int32) (*cms.ListModelsOutput, error)

	ListCMSItems(ctx context.Context, workspaceID, projectID, modelID string, keyword *string, page, pageSize *int32) (*cms.ListItemsOutput, error)

	GetCMSModelExportURL(ctx context.Context, workspaceID, projectID, modelID string, exportType *cms.ExportType) (string, error)
}
