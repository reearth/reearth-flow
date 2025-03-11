package interfaces

import (
	"context"
	"errors"

	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/account/accountdomain"
)

type CreateDeploymentParam struct {
	Project     *id.ProjectID
	Workspace   accountdomain.WorkspaceID
	Workflow    *file.File
	Description string
}

type UpdateDeploymentParam struct {
	ID          id.DeploymentID
	Workflow    *file.File
	Description *string
}

type ExecuteDeploymentParam struct {
	DeploymentID id.DeploymentID
}

var (
	ErrDeploymentNotFound error = errors.New("deployment not found")
	ErrJobCreationFailed  error = errors.New("failed to create job for deployment")
	ErrInvalidPagination  error = errors.New("invalid pagination parameters")
)

type Deployment interface {
	Fetch(context.Context, []id.DeploymentID) ([]*deployment.Deployment, error)
	FindByProject(context.Context, id.ProjectID) (*deployment.Deployment, error)
	FindByVersion(context.Context, accountdomain.WorkspaceID, *id.ProjectID, string) (*deployment.Deployment, error)
	FindByWorkspace(context.Context, accountdomain.WorkspaceID, *PaginationParam) ([]*deployment.Deployment, *PageBasedInfo, error)
	FindHead(context.Context, accountdomain.WorkspaceID, *id.ProjectID) (*deployment.Deployment, error)
	FindVersions(context.Context, accountdomain.WorkspaceID, *id.ProjectID) ([]*deployment.Deployment, error)
	Create(context.Context, CreateDeploymentParam) (*deployment.Deployment, error)
	Update(context.Context, UpdateDeploymentParam) (*deployment.Deployment, error)
	Delete(context.Context, id.DeploymentID) error
	Execute(context.Context, ExecuteDeploymentParam) (*job.Job, error)
}
