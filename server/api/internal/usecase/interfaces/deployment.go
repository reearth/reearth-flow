package interfaces

import (
	"context"
	"errors"

	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
)

type CreateDeploymentParam struct {
	Project     *id.ProjectID
	Workspace   id.WorkspaceID
	Workflow    *file.File
	Description string
	Variables   map[string]string
}

type UpdateDeploymentParam struct {
	ID          id.DeploymentID
	Workflow    *file.File
	Description *string
	Variables   map[string]string
}

type ExecuteDeploymentParam struct {
	DeploymentID id.DeploymentID
	Variables    map[string]string
}

var (
	ErrDeploymentNotFound    error = errors.New("deployment not found")
	ErrJobCreationFailed     error = errors.New("failed to create job for deployment")
	ErrInvalidPagination     error = errors.New("invalid pagination parameters")
	ErrDeploymentHasTriggers error = errors.New("deployment has active triggers and cannot be deleted")
)

type Deployment interface {
	Fetch(context.Context, []id.DeploymentID) ([]*deployment.Deployment, error)
	FindByProject(context.Context, id.ProjectID) (*deployment.Deployment, error)
	FindByVersion(context.Context, id.WorkspaceID, *id.ProjectID, string) (*deployment.Deployment, error)
	FindByWorkspace(context.Context, id.WorkspaceID, *PaginationParam, *string) ([]*deployment.Deployment, *PageBasedInfo, error)
	FindHead(context.Context, id.WorkspaceID, *id.ProjectID) (*deployment.Deployment, error)
	FindVersions(context.Context, id.WorkspaceID, *id.ProjectID) ([]*deployment.Deployment, error)
	Create(context.Context, CreateDeploymentParam) (*deployment.Deployment, error)
	Update(context.Context, UpdateDeploymentParam) (*deployment.Deployment, error)
	Delete(context.Context, id.DeploymentID) error
	Execute(context.Context, ExecuteDeploymentParam) (*job.Job, error)
}
