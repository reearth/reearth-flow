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

func (r *queryResolver) CmsProjects(ctx context.Context, workspaceID gqlmodel.ID, publicOnly *bool) ([]*gqlmodel.CMSProject, error) {
	projects, _, err := usecases(ctx).CMS.ListCMSProjects(ctx, string(workspaceID), lo.FromPtr(publicOnly))
	if err != nil {
		log.Errorfc(ctx, "failed to list CMS projects: %v", err)
		return nil, err
	}

	return util.Map(projects, gqlmodel.CMSProjectFrom), nil
}

func (r *queryResolver) CmsModels(ctx context.Context, projectID gqlmodel.ID) ([]*gqlmodel.CMSModel, error) {
	models, _, err := usecases(ctx).CMS.ListCMSModels(ctx, string(projectID))
	if err != nil {
		log.Errorfc(ctx, "failed to list CMS models: %v", err)
		return nil, err
	}

	return util.Map(models, gqlmodel.CMSModelFrom), nil
}

func (r *queryResolver) CmsItems(ctx context.Context, projectID gqlmodel.ID, modelID gqlmodel.ID, page *int, pageSize *int) (*gqlmodel.CMSItemsConnection, error) {
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

	output, err := usecases(ctx).CMS.ListCMSItems(ctx, string(projectID), string(modelID), pageInt32, pageSizeInt32)
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

func (r *queryResolver) CmsModelExportURL(ctx context.Context, projectID gqlmodel.ID, modelID gqlmodel.ID) (string, error) {
	url, err := usecases(ctx).CMS.GetCMSModelExportURL(ctx, string(projectID), string(modelID))
	if err != nil {
		log.Errorfc(ctx, "failed to get CMS model export URL: %v", err)
		return "", err
	}

	return url, nil
}
