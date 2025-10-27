package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/pkg/cms"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/util"
	"github.com/samber/lo"
)

func (r *queryResolver) CmsProject(ctx context.Context, projectIDOrAlias gqlmodel.ID) (*gqlmodel.CMSProject, error) {
	project, err := usecases(ctx).CMS.GetCMSProject(ctx, string(projectIDOrAlias))
	if err != nil {
		log.Errorfc(ctx, "failed to get CMS project: %v", err)
		return nil, err
	}
	if project == nil {
		return nil, nil
	}

	return gqlmodel.CMSProjectFrom(project), nil
}

func (r *queryResolver) CmsProjects(ctx context.Context, workspaceIDs []gqlmodel.ID, keyword *string, publicOnly *bool, page, pageSize *int) ([]*gqlmodel.CMSProject, error) {
	var pageInt32 *int32
	var pageSizeInt32 *int32

	if page != nil {
		v := int32(*page)
		pageInt32 = &v
	}
	if pageSize != nil {
		v := int32(*pageSize)
		pageSizeInt32 = &v
	}

	workspaceIDStrings := util.Map(workspaceIDs, func(id gqlmodel.ID) string { return string(id) })

	output, err := usecases(ctx).CMS.ListCMSProjects(ctx, workspaceIDStrings, keyword, lo.FromPtr(publicOnly), pageInt32, pageSizeInt32)
	if err != nil {
		log.Errorfc(ctx, "failed to list CMS projects: %v", err)
		return nil, err
	}

	return util.Map(output.Projects, gqlmodel.CMSProjectFrom), nil
}

func (r *queryResolver) CmsAsset(ctx context.Context, assetID gqlmodel.ID) (*gqlmodel.CMSAsset, error) {
	asset, err := usecases(ctx).CMS.GetCMSAsset(ctx, string(assetID))
	if err != nil {
		log.Errorfc(ctx, "failed to get CMS asset: %v", err)
		return nil, err
	}
	if asset == nil {
		return nil, nil
	}

	return gqlmodel.CMSAssetFrom(asset), nil
}

func (r *queryResolver) CmsAssets(ctx context.Context, projectID gqlmodel.ID, page, pageSize *int) (*gqlmodel.CMSAssetsConnection, error) {
	var pageInt32 *int32
	var pageSizeInt32 *int32

	if page != nil {
		v := int32(*page)
		pageInt32 = &v
	}
	if pageSize != nil {
		v := int32(*pageSize)
		pageSizeInt32 = &v
	}

	output, err := usecases(ctx).CMS.ListCMSAssets(ctx, string(projectID), pageInt32, pageSizeInt32)
	if err != nil {
		log.Errorfc(ctx, "failed to list CMS assets: %v", err)
		return nil, err
	}

	assets := util.Map(output.Assets, func(asset *cms.Asset) *gqlmodel.CMSAsset {
		return gqlmodel.CMSAssetFrom(asset)
	})

	pageInfo := &gqlmodel.CMSPageInfo{}
	if output.PageInfo != nil {
		pageInfo.Page = int(output.PageInfo.Page)
		pageInfo.PageSize = int(output.PageInfo.PageSize)
	}

	return &gqlmodel.CMSAssetsConnection{
		Assets:     assets,
		TotalCount: int(output.TotalCount),
		PageInfo:   pageInfo,
	}, nil
}

func (r *queryResolver) CmsModel(ctx context.Context, projectIDOrAlias gqlmodel.ID, modelIDOrAlias gqlmodel.ID) (*gqlmodel.CMSModel, error) {
	model, err := usecases(ctx).CMS.GetCMSModel(ctx, string(projectIDOrAlias), string(modelIDOrAlias))
	if err != nil {
		log.Errorfc(ctx, "failed to get CMS model: %v", err)
		return nil, err
	}
	if model == nil {
		return nil, nil
	}

	return gqlmodel.CMSModelFrom(model), nil
}

func (r *queryResolver) CmsModels(ctx context.Context, projectID gqlmodel.ID, page, pageSize *int) (*gqlmodel.CMSModelsConnection, error) {
	var pageInt32 *int32
	var pageSizeInt32 *int32

	if page != nil {
		v := int32(*page)
		pageInt32 = &v
	}
	if pageSize != nil {
		v := int32(*pageSize)
		pageSizeInt32 = &v
	}

	output, err := usecases(ctx).CMS.ListCMSModels(ctx, string(projectID), pageInt32, pageSizeInt32)
	if err != nil {
		log.Errorfc(ctx, "failed to list CMS models: %v", err)
		return nil, err
	}

	models := util.Map(output.Models, func(model *cms.Model) *gqlmodel.CMSModel {
		return gqlmodel.CMSModelFrom(model)
	})

	pageInfo := &gqlmodel.CMSPageInfo{}
	if output.PageInfo != nil {
		pageInfo.Page = int(output.PageInfo.Page)
		pageInfo.PageSize = int(output.PageInfo.PageSize)
	}

	return &gqlmodel.CMSModelsConnection{
		Models:     models,
		TotalCount: int(output.TotalCount),
		PageInfo:   pageInfo,
	}, nil
}

func (r *queryResolver) CmsItems(ctx context.Context, projectID gqlmodel.ID, modelID gqlmodel.ID, keyword *string, page *int, pageSize *int) (*gqlmodel.CMSItemsConnection, error) {
	var pageInt32 *int32
	var pageSizeInt32 *int32

	if page != nil {
		v := int32(*page)
		pageInt32 = &v
	}
	if pageSize != nil {
		v := int32(*pageSize)
		pageSizeInt32 = &v
	}

	output, err := usecases(ctx).CMS.ListCMSItems(ctx, string(projectID), string(modelID), keyword, pageInt32, pageSizeInt32)
	if err != nil {
		log.Errorfc(ctx, "failed to list CMS items: %v", err)
		return nil, err
	}

	items := util.Map(output.Items, func(item cms.Item) *gqlmodel.CMSItem {
		return gqlmodel.CMSItemFrom(&item)
	})

	return &gqlmodel.CMSItemsConnection{
		Items:      items,
		TotalCount: int(output.TotalCount),
	}, nil
}

func (r *queryResolver) CmsModelExportURL(ctx context.Context, projectID gqlmodel.ID, modelID gqlmodel.ID, exportType *gqlmodel.CMSExportType) (string, error) {
	var cmsExportType *cms.ExportType
	if exportType != nil {
		switch *exportType {
		case gqlmodel.CMSExportTypeJSON:
			t := cms.ExportTypeJSON
			cmsExportType = &t
		case gqlmodel.CMSExportTypeGeojson:
			t := cms.ExportTypeGeoJSON
			cmsExportType = &t
		}
	}

	url, err := usecases(ctx).CMS.GetCMSModelExportURL(ctx, string(projectID), string(modelID), cmsExportType)
	if err != nil {
		log.Errorfc(ctx, "failed to get CMS model export URL: %v", err)
		return "", err
	}

	return url, nil
}
