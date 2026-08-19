package interactor

import (
	"context"
	"fmt"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/cms"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/rerror"
)

var _ interfaces.CMS = (*cmsInteractor)(nil)

type cmsInteractor struct {
	repos             *repo.Container
	gateways          *gateway.Container
	permissionChecker gateway.PermissionChecker
}

func NewCMS(r *repo.Container, gr *gateway.Container, permissionChecker gateway.PermissionChecker) interfaces.CMS {
	return &cmsInteractor{
		repos:             r,
		gateways:          gr,
		permissionChecker: permissionChecker,
	}
}

// workspaceOfCMSProject resolves the workspace that owns projectIDOrAlias.
// Never cache this on the interactor: the container is per-request today but
// is becoming a boot-time singleton, and struct-level state here would turn
// into a process-wide cross-tenant cache.
func (i *cmsInteractor) workspaceOfCMSProject(ctx context.Context, projectIDOrAlias string) (accountsid.WorkspaceID, error) {
	project, err := i.gateways.CMS.GetProject(ctx, projectIDOrAlias)
	if err != nil {
		return accountsid.WorkspaceID{}, err
	}
	if project == nil {
		return accountsid.WorkspaceID{}, rerror.ErrNotFound
	}
	wsID, err := accountsid.WorkspaceIDFrom(project.WorkspaceID)
	if err != nil {
		return accountsid.WorkspaceID{}, rerror.ErrNotFound
	}
	return wsID, nil
}

func (i *cmsInteractor) GetCMSProject(ctx context.Context, projectIDOrAlias string) (*cms.Project, error) {
	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
	}

	log.Debugfc(ctx, "Fetching CMS project: %s", projectIDOrAlias)

	project, err := i.gateways.CMS.GetProject(ctx, projectIDOrAlias)
	if err != nil {
		return nil, fmt.Errorf("failed to get CMS project: %w", err)
	}
	if project == nil {
		return nil, rerror.ErrNotFound
	}

	wsID, err := accountsid.WorkspaceIDFrom(project.WorkspaceID)
	if err != nil {
		return nil, rerror.ErrNotFound
	}
	if err := checkPermission(ctx, i.permissionChecker, rbac.ResourceCMSProject, rbac.ActionAny, wsID); err != nil {
		return nil, rerror.ErrNotFound
	}

	return project, nil
}

func (i *cmsInteractor) ListCMSProjects(ctx context.Context, workspaceIDs []string, keyword *string, publicOnly bool, page, pageSize *int32) (*cms.ListProjectsOutput, error) {
	// Fail closed: every requested workspace must pass on its own. The client
	// is expected to only ask for workspaces it believes it can read.
	for _, wsIDStr := range workspaceIDs {
		wsID, err := accountsid.WorkspaceIDFrom(wsIDStr)
		if err != nil {
			return nil, rerror.ErrNotFound
		}
		if err := checkPermission(ctx, i.permissionChecker, rbac.ResourceCMSProject, rbac.ActionAny, wsID); err != nil {
			return nil, rerror.ErrNotFound
		}
	}

	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
	}

	log.Debugfc(ctx, "Listing CMS projects for workspaces: %v, keyword: %v, publicOnly: %v", workspaceIDs, keyword, publicOnly)

	var pageInfo *cms.PageInfo
	if page != nil && pageSize != nil {
		pageInfo = &cms.PageInfo{
			Page:     *page,
			PageSize: *pageSize,
		}
	}

	return i.gateways.CMS.ListProjects(ctx, cms.ListProjectsInput{
		WorkspaceIDs: workspaceIDs,
		Keyword:      keyword,
		PublicOnly:   publicOnly,
		PageInfo:     pageInfo,
	})
}

func (i *cmsInteractor) GetCMSAsset(ctx context.Context, assetID string) (*cms.Asset, error) {
	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
	}

	log.Debugfc(ctx, "Fetching CMS asset: %s", assetID)

	asset, err := i.gateways.CMS.GetAsset(ctx, cms.GetAssetInput{
		AssetID: assetID,
	})
	if err != nil {
		return nil, err
	}
	if asset == nil {
		return nil, rerror.ErrNotFound
	}

	ws, err := i.workspaceOfCMSProject(ctx, asset.ProjectID)
	if err != nil {
		return nil, err
	}
	if err := checkPermission(ctx, i.permissionChecker, rbac.ResourceCMSAsset, rbac.ActionAny, ws); err != nil {
		return nil, rerror.ErrNotFound
	}

	return asset, nil
}

func (i *cmsInteractor) ListCMSAssets(ctx context.Context, projectID string, page, pageSize *int32) (*cms.ListAssetsOutput, error) {
	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
	}

	ws, err := i.workspaceOfCMSProject(ctx, projectID)
	if err != nil {
		return nil, err
	}
	if err := checkPermission(ctx, i.permissionChecker, rbac.ResourceCMSAsset, rbac.ActionAny, ws); err != nil {
		return nil, rerror.ErrNotFound
	}

	log.Debugfc(ctx, "Listing CMS assets for project: %s", projectID)

	var pageInfo *cms.PageInfo
	if page != nil && pageSize != nil {
		pageInfo = &cms.PageInfo{
			Page:     *page,
			PageSize: *pageSize,
		}
	}

	return i.gateways.CMS.ListAssets(ctx, cms.ListAssetsInput{
		ProjectID: projectID,
		PageInfo:  pageInfo,
	})
}

func (i *cmsInteractor) GetCMSModel(ctx context.Context, projectIDOrAlias, modelIDOrAlias string) (*cms.Model, error) {
	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
	}

	ws, err := i.workspaceOfCMSProject(ctx, projectIDOrAlias)
	if err != nil {
		return nil, err
	}
	if err := checkPermission(ctx, i.permissionChecker, rbac.ResourceCMSModel, rbac.ActionAny, ws); err != nil {
		return nil, rerror.ErrNotFound
	}

	log.Debugfc(ctx, "Fetching CMS model: %s in project: %s", modelIDOrAlias, projectIDOrAlias)

	return i.gateways.CMS.GetModel(ctx, cms.GetModelInput{
		ProjectIDOrAlias: projectIDOrAlias,
		ModelIDOrAlias:   modelIDOrAlias,
	})
}

func (i *cmsInteractor) ListCMSModels(ctx context.Context, projectID string, page, pageSize *int32) (*cms.ListModelsOutput, error) {
	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
	}

	ws, err := i.workspaceOfCMSProject(ctx, projectID)
	if err != nil {
		return nil, err
	}
	if err := checkPermission(ctx, i.permissionChecker, rbac.ResourceCMSModel, rbac.ActionAny, ws); err != nil {
		return nil, rerror.ErrNotFound
	}

	log.Debugfc(ctx, "Listing CMS models for project: %s", projectID)

	var pageInfo *cms.PageInfo
	if page != nil && pageSize != nil {
		pageInfo = &cms.PageInfo{
			Page:     *page,
			PageSize: *pageSize,
		}
	}

	return i.gateways.CMS.ListModels(ctx, cms.ListModelsInput{
		ProjectID: projectID,
		PageInfo:  pageInfo,
	})
}

func (i *cmsInteractor) ListCMSItems(ctx context.Context, projectID, modelID string, keyword *string, page, pageSize *int32) (*cms.ListItemsOutput, error) {
	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
	}

	ws, err := i.workspaceOfCMSProject(ctx, projectID)
	if err != nil {
		return nil, err
	}
	if err := checkPermission(ctx, i.permissionChecker, rbac.ResourceCMSItem, rbac.ActionAny, ws); err != nil {
		return nil, rerror.ErrNotFound
	}

	log.Debugfc(ctx, "Listing CMS items for model: %s in project: %s", modelID, projectID)

	var pageInfo *cms.PageInfo
	if page != nil && pageSize != nil {
		pageInfo = &cms.PageInfo{
			Page:     *page,
			PageSize: *pageSize,
		}
	}

	return i.gateways.CMS.ListItems(ctx, cms.ListItemsInput{
		ProjectID: projectID,
		ModelID:   modelID,
		Keyword:   keyword,
		PageInfo:  pageInfo,
	})
}

func (i *cmsInteractor) GetCMSModelExportURL(ctx context.Context, projectID, modelID string, exportType *cms.ExportType) (string, error) {
	if i.gateways.CMS == nil {
		return "", fmt.Errorf("CMS gateway not configured")
	}

	ws, err := i.workspaceOfCMSProject(ctx, projectID)
	if err != nil {
		return "", err
	}
	if err := checkPermission(ctx, i.permissionChecker, rbac.ResourceCMSModel, rbac.ActionAny, ws); err != nil {
		return "", rerror.ErrNotFound
	}

	if exportType != nil {
		output, err := i.gateways.CMS.GetModelExportURL(ctx, cms.ModelExportInput{
			ProjectID:  projectID,
			ModelID:    modelID,
			ExportType: *exportType,
		})
		if err != nil {
			return "", err
		}
		return output.URL, nil
	}

	output, err := i.gateways.CMS.GetModelGeoJSONExportURL(ctx, cms.ExportInput{
		ProjectID: projectID,
		ModelID:   modelID,
	})
	if err != nil {
		return "", err
	}

	return output.URL, nil
}
