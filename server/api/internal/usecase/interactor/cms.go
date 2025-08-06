package interactor

import (
	"context"
	"fmt"

	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/cms"
	"github.com/reearth/reearthx/log"
)

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
	op := adapter.Operator(ctx)
	if op == nil {
		return nil, fmt.Errorf("operator not found")
	}

	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
	}

	log.Debugfc(ctx, "Fetching CMS project: %s for user: %s", projectIDOrAlias, op.AcOperator.User)

	project, err := i.gateways.CMS.GetProject(ctx, projectIDOrAlias)
	if err != nil {
		return nil, fmt.Errorf("failed to get CMS project: %w", err)
	}

	// authInfo := adapter.GetAuthInfo(ctx)
	// allowed, err := i.permissionChecker.CheckPermission(ctx, authInfo, op.AcOperator.User.String(),
	// 	fmt.Sprintf("workspace:%s", project.WorkspaceID), "read")
	// if err != nil {
	// 	return nil, fmt.Errorf("failed to check permission: %w", err)
	// }
	// if !allowed {
	// 	return nil, fmt.Errorf("permission denied: cannot access workspace %s", project.WorkspaceID)
	// }

	return project, nil
}

func (i *cmsInteractor) ListCMSProjects(ctx context.Context, workspaceID string, publicOnly bool) ([]*cms.Project, int32, error) {
	op := adapter.Operator(ctx)
	if op == nil {
		return nil, 0, fmt.Errorf("operator not found")
	}

	if i.gateways.CMS == nil {
		return nil, 0, fmt.Errorf("CMS gateway not configured")
	}

	// if !publicOnly {
	// 	authInfo := adapter.GetAuthInfo(ctx)
	// 	allowed, err := i.permissionChecker.CheckPermission(ctx, authInfo, op.AcOperator.User.String(),
	// 		fmt.Sprintf("workspace:%s", workspaceID), "read")
	// 	if err != nil {
	// 		return nil, 0, fmt.Errorf("failed to check permission: %w", err)
	// 	}
	// 	if !allowed {
	// 		return nil, 0, fmt.Errorf("permission denied: cannot access workspace %s", workspaceID)
	// 	}
	// }

	log.Debugfc(ctx, "Listing CMS projects for workspace: %s, publicOnly: %v", workspaceID, publicOnly)

	return i.gateways.CMS.ListProjects(ctx, cms.ListProjectsInput{
		WorkspaceID: workspaceID,
		PublicOnly:  publicOnly,
	})
}

func (i *cmsInteractor) ListCMSModels(ctx context.Context, projectID string) ([]*cms.Model, int32, error) {
	op := adapter.Operator(ctx)
	if op == nil {
		return nil, 0, fmt.Errorf("operator not found")
	}

	if i.gateways.CMS == nil {
		return nil, 0, fmt.Errorf("CMS gateway not configured")
	}

	// project, err := i.gateways.CMS.GetProject(ctx, projectID)
	// if err != nil {
	// 	return nil, 0, fmt.Errorf("failed to get CMS project: %w", err)
	// }

	// authInfo := adapter.GetAuthInfo(ctx)
	// allowed, err := i.permissionChecker.CheckPermission(ctx, authInfo, op.AcOperator.User.String(),
	// 	fmt.Sprintf("workspace:%s", project.WorkspaceID), "read")
	// if err != nil {
	// 	return nil, 0, fmt.Errorf("failed to check permission: %w", err)
	// }
	// if !allowed {
	// 	return nil, 0, fmt.Errorf("permission denied: cannot access workspace %s", project.WorkspaceID)
	// }

	log.Debugfc(ctx, "Listing CMS models for project: %s", projectID)

	return i.gateways.CMS.ListModels(ctx, cms.ListModelsInput{
		ProjectID: projectID,
	})
}

func (i *cmsInteractor) ListCMSItems(ctx context.Context, projectID, modelID string, keyword *string, page, pageSize *int32) (*cms.ListItemsOutput, error) {
	op := adapter.Operator(ctx)
	if op == nil {
		return nil, fmt.Errorf("operator not found")
	}

	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
	}

	// project, err := i.gateways.CMS.GetProject(ctx, projectID)
	// if err != nil {
	// 	return nil, fmt.Errorf("failed to get CMS project: %w", err)
	// }

	// authInfo := adapter.GetAuthInfo(ctx)
	// allowed, err := i.permissionChecker.CheckPermission(ctx, authInfo, op.AcOperator.User.String(),
	// 	fmt.Sprintf("workspace:%s", project.WorkspaceID), "read")
	// if err != nil {
	// 	return nil, fmt.Errorf("failed to check permission: %w", err)
	// }
	// if !allowed {
	// 	return nil, fmt.Errorf("permission denied: cannot access workspace %s", project.WorkspaceID)
	// }

	log.Debugfc(ctx, "Listing CMS items for model: %s in project: %s", modelID, projectID)

	return i.gateways.CMS.ListItems(ctx, cms.ListItemsInput{
		ProjectID: projectID,
		ModelID:   modelID,
		Keyword:   keyword,
		Page:      page,
		PageSize:  pageSize,
	})
}

func (i *cmsInteractor) GetCMSModelExportURL(ctx context.Context, projectID, modelID string) (string, error) {
	op := adapter.Operator(ctx)
	if op == nil {
		return "", fmt.Errorf("operator not found")
	}

	if i.gateways.CMS == nil {
		return "", fmt.Errorf("CMS gateway not configured")
	}

	// project, err := i.gateways.CMS.GetProject(ctx, projectID)
	// if err != nil {
	// 	return "", fmt.Errorf("failed to get CMS project: %w", err)
	// }

	// authInfo := adapter.GetAuthInfo(ctx)
	// allowed, err := i.permissionChecker.CheckPermission(ctx, authInfo, op.AcOperator.User.String(),
	// 	fmt.Sprintf("workspace:%s", project.WorkspaceID), "read")
	// if err != nil {
	// 	return "", fmt.Errorf("failed to check permission: %w", err)
	// }
	// if !allowed {
	// 	return "", fmt.Errorf("permission denied: cannot access workspace %s", project.WorkspaceID)
	// }

	// log.Debugfc(ctx, "Getting CMS model export URL for model: %s in project: %s", modelID, projectID)

	output, err := i.gateways.CMS.GetModelGeoJSONExportURL(ctx, cms.ExportInput{
		ProjectID: projectID,
		ModelID:   modelID,
	})
	if err != nil {
		return "", err
	}

	return output.URL, nil
}
