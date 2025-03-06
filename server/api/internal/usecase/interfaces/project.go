package interfaces

import (
	"context"
	"errors"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/account/accountdomain"
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
	Workflow  *file.File
}

var (
	ErrProjectAliasIsNotSet    error = errors.New("project alias is not set")
	ErrProjectAliasAlreadyUsed error = errors.New("project alias is already used by another project")
)

type Project interface {
	Fetch(context.Context, []id.ProjectID, *usecase.Operator) ([]*project.Project, error)
	FindByWorkspace(context.Context, accountdomain.WorkspaceID, *PaginationParam, *usecase.Operator) ([]*project.Project, *PageBasedInfo, error)
	Create(context.Context, CreateProjectParam, *usecase.Operator) (*project.Project, error)
	Update(context.Context, UpdateProjectParam, *usecase.Operator) (*project.Project, error)
	Delete(context.Context, id.ProjectID, *usecase.Operator) error
	Run(context.Context, RunProjectParam, *usecase.Operator) (*job.Job, error)
}
