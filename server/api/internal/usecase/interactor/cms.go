package interactor

import (
	"context"
	"fmt"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/cms"
	"github.com/reearth/reearthx/log"
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

func (i *cmsInteractor) GetCMSProject(ctx context.Context, projectIDOrAlias string) (*cms.Project, error) {
	if err := checkPermission(ctx, i.permissionChecker, rbac.ResourceCMSProject, rbac.ActionAny); err != nil {
		return nil, err
	}

	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
	}

	log.Debugfc(ctx, "Fetching CMS project: %s", projectIDOrAlias)

	project, err := i.gateways.CMS.GetProject(ctx, projectIDOrAlias)
	if err != nil {
		return nil, fmt.Errorf("failed to get CMS project: %w", err)
	}

	return project, nil
}

func (i *cmsInteractor) ListCMSProjects(ctx context.Context, workspaceIDs []string, publicOnly bool, page, pageSize *int32) (*cms.ListProjectsOutput, error) {
	if err := checkPermission(ctx, i.permissionChecker, rbac.ResourceCMSProject, rbac.ActionAny); err != nil {
		return nil, err
	}

	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
	}

	log.Debugfc(ctx, "Listing CMS projects for workspaces: %v, publicOnly: %v", workspaceIDs, publicOnly)

	var pageInfo *cms.PageInfo
	if page != nil && pageSize != nil {
		pageInfo = &cms.PageInfo{
			Page:     *page,
			PageSize: *pageSize,
		}
	}

	return i.gateways.CMS.ListProjects(ctx, cms.ListProjectsInput{
		WorkspaceIDs: workspaceIDs,
		PublicOnly:   publicOnly,
		PageInfo:     pageInfo,
	})
}

func (i *cmsInteractor) GetCMSAsset(ctx context.Context, assetID string) (*cms.Asset, error) {
	if err := checkPermission(ctx, i.permissionChecker, rbac.ResourceCMSAsset, rbac.ActionAny); err != nil {
		return nil, err
	}

	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
	}

	log.Debugfc(ctx, "Fetching CMS asset: %s", assetID)

	return i.gateways.CMS.GetAsset(ctx, cms.GetAssetInput{
		AssetID: assetID,
	})
}

func (i *cmsInteractor) ListCMSAssets(ctx context.Context, projectID string, page, pageSize *int32) (*cms.ListAssetsOutput, error) {
	if err := checkPermission(ctx, i.permissionChecker, rbac.ResourceCMSAsset, rbac.ActionAny); err != nil {
		return nil, err
	}

	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
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
	if err := checkPermission(ctx, i.permissionChecker, rbac.ResourceCMSModel, rbac.ActionAny); err != nil {
		return nil, err
	}

	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
	}

	log.Debugfc(ctx, "Fetching CMS model: %s in project: %s", modelIDOrAlias, projectIDOrAlias)

	return i.gateways.CMS.GetModel(ctx, cms.GetModelInput{
		ProjectIDOrAlias: projectIDOrAlias,
		ModelIDOrAlias:   modelIDOrAlias,
	})
}

func (i *cmsInteractor) ListCMSModels(ctx context.Context, projectID string, page, pageSize *int32) (*cms.ListModelsOutput, error) {
	if err := checkPermission(ctx, i.permissionChecker, rbac.ResourceCMSModel, rbac.ActionAny); err != nil {
		return nil, err
	}

	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
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
	if err := checkPermission(ctx, i.permissionChecker, rbac.ResourceCMSItem, rbac.ActionAny); err != nil {
		return nil, err
	}

	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
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

func (i *cmsInteractor) GetCMSModelExportURL(ctx context.Context, projectID, modelID string) (string, error) {
	if err := checkPermission(ctx, i.permissionChecker, rbac.ResourceCMSModel, rbac.ActionAny); err != nil {
		return "", err
	}

	if i.gateways.CMS == nil {
		return "", fmt.Errorf("CMS gateway not configured")
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
