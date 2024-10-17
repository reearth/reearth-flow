package interfaces

import (
	"context"
	"errors"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/usecasex"
)

type CreateProjectParam struct {
	WorkspaceID accountdomain.WorkspaceID
	Name        *string
	Description *string
	Archived    *bool
}

type UpdateProjectParam struct {
	ID                id.ProjectID
	Name              *string
	Description       *string
	Archived          *bool
	IsBasicAuthActive *bool
	BasicAuthUsername *string
	BasicAuthPassword *string
}

type RunProjectParam struct {
	ProjectID id.ProjectID
	Meta      *file.File
	Workflow  *file.File
}

var (
	ErrProjectAliasIsNotSet    error = errors.New("project alias is not set")
	ErrProjectAliasAlreadyUsed error = errors.New("project alias is already used by another project")
)

type Project interface {
	Fetch(context.Context, []id.ProjectID, *usecase.Operator) ([]*project.Project, error)
	FindByWorkspace(context.Context, accountdomain.WorkspaceID, *usecasex.Pagination, *usecase.Operator) ([]*project.Project, *usecasex.PageInfo, error)
	Create(context.Context, CreateProjectParam, *usecase.Operator) (*project.Project, error)
	Update(context.Context, UpdateProjectParam, *usecase.Operator) (*project.Project, error)
	Delete(context.Context, id.ProjectID, *usecase.Operator) error
	Run(context.Context, RunProjectParam, *usecase.Operator) (bool, error)
}
