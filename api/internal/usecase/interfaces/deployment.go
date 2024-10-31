package interfaces

import (
	"context"
	"errors"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/usecasex"
)

type CreateDeploymentParam struct {
	Project     id.ProjectID
	Workspace   accountdomain.WorkspaceID
	Workflow    *file.File
	Description *string
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
)

type Deployment interface {
	Fetch(context.Context, []id.DeploymentID, *usecase.Operator) ([]*deployment.Deployment, error)
	FindByWorkspace(context.Context, accountdomain.WorkspaceID, *usecasex.Pagination, *usecase.Operator) ([]*deployment.Deployment, *usecasex.PageInfo, error)
	FindByProject(context.Context, id.ProjectID, *usecase.Operator) (*deployment.Deployment, error)
	Create(context.Context, CreateDeploymentParam, *usecase.Operator) (*deployment.Deployment, error)
	Update(context.Context, UpdateDeploymentParam, *usecase.Operator) (*deployment.Deployment, error)
	Delete(context.Context, id.DeploymentID, *usecase.Operator) error
	Execute(context.Context, ExecuteDeploymentParam, *usecase.Operator) (*job.Job, error)
}
