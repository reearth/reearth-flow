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

// NewCMS creates a new CMS interactor
func NewCMS(r *repo.Container, gr *gateway.Container, permissionChecker gateway.PermissionChecker) interfaces.CMS {
	return &cmsInteractor{
		repos:             r,
		gateways:          gr,
		permissionChecker: permissionChecker,
	}
}

// GetCMSProject retrieves a CMS project by ID or alias
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

	// Check if user has access to the workspace
	authInfo := adapter.GetAuthInfo(ctx)
	allowed, err := i.permissionChecker.CheckPermission(ctx, authInfo, op.AcOperator.User.String(),
		fmt.Sprintf("workspace:%s", project.WorkspaceID), "read")
	if err != nil {
		return nil, fmt.Errorf("failed to check permission: %w", err)
	}
	if !allowed {
		return nil, fmt.Errorf("permission denied: cannot access workspace %s", project.WorkspaceID)
	}

	return project, nil
}

// ListCMSProjects lists CMS projects for a workspace
func (i *cmsInteractor) ListCMSProjects(ctx context.Context, workspaceID string, publicOnly bool) ([]*cms.Project, int32, error) {
	op := adapter.Operator(ctx)
	if op == nil {
		return nil, 0, fmt.Errorf("operator not found")
	}

	if i.gateways.CMS == nil {
		return nil, 0, fmt.Errorf("CMS gateway not configured")
	}

	// Check if user has access to the workspace
	if !publicOnly {
		authInfo := adapter.GetAuthInfo(ctx)
		allowed, err := i.permissionChecker.CheckPermission(ctx, authInfo, op.AcOperator.User.String(),
			fmt.Sprintf("workspace:%s", workspaceID), "read")
		if err != nil {
			return nil, 0, fmt.Errorf("failed to check permission: %w", err)
		}
		if !allowed {
			return nil, 0, fmt.Errorf("permission denied: cannot access workspace %s", workspaceID)
		}
	}

	log.Debugfc(ctx, "Listing CMS projects for workspace: %s, publicOnly: %v", workspaceID, publicOnly)

	return i.gateways.CMS.ListProjects(ctx, cms.ListProjectsInput{
		WorkspaceID: workspaceID,
		PublicOnly:  publicOnly,
	})
}

// ListCMSModels lists models for a CMS project
func (i *cmsInteractor) ListCMSModels(ctx context.Context, projectID string) ([]*cms.Model, int32, error) {
	op := adapter.Operator(ctx)
	if op == nil {
		return nil, 0, fmt.Errorf("operator not found")
	}

	if i.gateways.CMS == nil {
		return nil, 0, fmt.Errorf("CMS gateway not configured")
	}

	// First get the project to check permissions
	project, err := i.gateways.CMS.GetProject(ctx, projectID)
	if err != nil {
		return nil, 0, fmt.Errorf("failed to get CMS project: %w", err)
	}

	// Check if user has access to the workspace
	authInfo := adapter.GetAuthInfo(ctx)
	allowed, err := i.permissionChecker.CheckPermission(ctx, authInfo, op.AcOperator.User.String(),
		fmt.Sprintf("workspace:%s", project.WorkspaceID), "read")
	if err != nil {
		return nil, 0, fmt.Errorf("failed to check permission: %w", err)
	}
	if !allowed {
		return nil, 0, fmt.Errorf("permission denied: cannot access workspace %s", project.WorkspaceID)
	}

	log.Debugfc(ctx, "Listing CMS models for project: %s", projectID)

	return i.gateways.CMS.ListModels(ctx, cms.ListModelsInput{
		ProjectID: projectID,
	})
}

// ListCMSItems lists items for a CMS model
func (i *cmsInteractor) ListCMSItems(ctx context.Context, projectID, modelID string, page, pageSize *int32) (*cms.ListItemsOutput, error) {
	op := adapter.Operator(ctx)
	if op == nil {
		return nil, fmt.Errorf("operator not found")
	}

	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
	}

	// First get the project to check permissions
	project, err := i.gateways.CMS.GetProject(ctx, projectID)
	if err != nil {
		return nil, fmt.Errorf("failed to get CMS project: %w", err)
	}

	// Check if user has access to the workspace
	authInfo := adapter.GetAuthInfo(ctx)
	allowed, err := i.permissionChecker.CheckPermission(ctx, authInfo, op.AcOperator.User.String(),
		fmt.Sprintf("workspace:%s", project.WorkspaceID), "read")
	if err != nil {
		return nil, fmt.Errorf("failed to check permission: %w", err)
	}
	if !allowed {
		return nil, fmt.Errorf("permission denied: cannot access workspace %s", project.WorkspaceID)
	}

	log.Debugfc(ctx, "Listing CMS items for model: %s in project: %s", modelID, projectID)

	return i.gateways.CMS.ListItems(ctx, cms.ListItemsInput{
		ProjectID: projectID,
		ModelID:   modelID,
		Page:      page,
		PageSize:  pageSize,
	})
}

// GetCMSModelExportURL gets the GeoJSON export URL for a CMS model
func (i *cmsInteractor) GetCMSModelExportURL(ctx context.Context, projectID, modelID string) (string, error) {
	op := adapter.Operator(ctx)
	if op == nil {
		return "", fmt.Errorf("operator not found")
	}

	if i.gateways.CMS == nil {
		return "", fmt.Errorf("CMS gateway not configured")
	}

	// First get the project to check permissions
	project, err := i.gateways.CMS.GetProject(ctx, projectID)
	if err != nil {
		return "", fmt.Errorf("failed to get CMS project: %w", err)
	}

	// Check if user has access to the workspace
	authInfo := adapter.GetAuthInfo(ctx)
	allowed, err := i.permissionChecker.CheckPermission(ctx, authInfo, op.AcOperator.User.String(),
		fmt.Sprintf("workspace:%s", project.WorkspaceID), "read")
	if err != nil {
		return "", fmt.Errorf("failed to check permission: %w", err)
	}
	if !allowed {
		return "", fmt.Errorf("permission denied: cannot access workspace %s", project.WorkspaceID)
	}

	log.Debugfc(ctx, "Getting CMS model export URL for model: %s in project: %s", modelID, projectID)

	output, err := i.gateways.CMS.GetModelGeoJSONExportURL(ctx, cms.ExportInput{
		ProjectID: projectID,
		ModelID:   modelID,
	})
	if err != nil {
		return "", err
	}

	return output.URL, nil
}

// CreateCMSProject creates a new CMS project
func (i *cmsInteractor) CreateCMSProject(ctx context.Context, input cms.CreateProjectInput) (*cms.Project, error) {
	op := adapter.Operator(ctx)
	if op == nil {
		return nil, fmt.Errorf("operator not found")
	}

	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
	}

	// Check if user has write access to the workspace
	authInfo := adapter.GetAuthInfo(ctx)
	allowed, err := i.permissionChecker.CheckPermission(ctx, authInfo, op.AcOperator.User.String(),
		fmt.Sprintf("workspace:%s", input.WorkspaceID), "write")
	if err != nil {
		return nil, fmt.Errorf("failed to check permission: %w", err)
	}
	if !allowed {
		return nil, fmt.Errorf("permission denied: cannot create project in workspace %s", input.WorkspaceID)
	}

	log.Debugfc(ctx, "Creating CMS project: %s in workspace: %s for user: %s", input.Name, input.WorkspaceID, op.AcOperator.User)

	return i.gateways.CMS.CreateProject(ctx, input)
}

// UpdateCMSProject updates an existing CMS project
func (i *cmsInteractor) UpdateCMSProject(ctx context.Context, input cms.UpdateProjectInput) (*cms.Project, error) {
	op := adapter.Operator(ctx)
	if op == nil {
		return nil, fmt.Errorf("operator not found")
	}

	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
	}

	// First get the project to check permissions
	project, err := i.gateways.CMS.GetProject(ctx, input.ProjectID)
	if err != nil {
		return nil, fmt.Errorf("failed to get CMS project: %w", err)
	}

	// Check if user has write access to the workspace
	authInfo := adapter.GetAuthInfo(ctx)
	allowed, err := i.permissionChecker.CheckPermission(ctx, authInfo, op.AcOperator.User.String(),
		fmt.Sprintf("workspace:%s", project.WorkspaceID), "write")
	if err != nil {
		return nil, fmt.Errorf("failed to check permission: %w", err)
	}
	if !allowed {
		return nil, fmt.Errorf("permission denied: cannot update project in workspace %s", project.WorkspaceID)
	}

	log.Debugfc(ctx, "Updating CMS project: %s for user: %s", input.ProjectID, op.AcOperator.User)

	return i.gateways.CMS.UpdateProject(ctx, input)
}

// DeleteCMSProject deletes a CMS project
func (i *cmsInteractor) DeleteCMSProject(ctx context.Context, input cms.DeleteProjectInput) (*cms.DeleteProjectOutput, error) {
	op := adapter.Operator(ctx)
	if op == nil {
		return nil, fmt.Errorf("operator not found")
	}

	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
	}

	// First get the project to check permissions
	project, err := i.gateways.CMS.GetProject(ctx, input.ProjectID)
	if err != nil {
		return nil, fmt.Errorf("failed to get CMS project: %w", err)
	}

	// Check if user has write access to the workspace
	authInfo := adapter.GetAuthInfo(ctx)
	allowed, err := i.permissionChecker.CheckPermission(ctx, authInfo, op.AcOperator.User.String(),
		fmt.Sprintf("workspace:%s", project.WorkspaceID), "write")
	if err != nil {
		return nil, fmt.Errorf("failed to check permission: %w", err)
	}
	if !allowed {
		return nil, fmt.Errorf("permission denied: cannot delete project in workspace %s", project.WorkspaceID)
	}

	log.Debugfc(ctx, "Deleting CMS project: %s for user: %s", input.ProjectID, op.AcOperator.User)

	return i.gateways.CMS.DeleteProject(ctx, input)
}

// CheckCMSAliasAvailability checks if a project alias is available
func (i *cmsInteractor) CheckCMSAliasAvailability(ctx context.Context, input cms.CheckAliasAvailabilityInput) (*cms.CheckAliasAvailabilityOutput, error) {
	op := adapter.Operator(ctx)
	if op == nil {
		return nil, fmt.Errorf("operator not found")
	}

	if i.gateways.CMS == nil {
		return nil, fmt.Errorf("CMS gateway not configured")
	}

	log.Debugfc(ctx, "Checking CMS alias availability: %s for user: %s", input.Alias, op.AcOperator.User)

	return i.gateways.CMS.CheckAliasAvailability(ctx, input)
}
