package interfaces

import (
	"context"
	"errors"

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
	Fetch(context.Context, []id.ProjectID) ([]*project.Project, error)
	FindByWorkspace(context.Context, accountdomain.WorkspaceID, *PaginationParam) ([]*project.Project, *PageBasedInfo, error)
	Create(context.Context, CreateProjectParam) (*project.Project, error)
	Update(context.Context, UpdateProjectParam) (*project.Project, error)
	Delete(context.Context, id.ProjectID) error
	Run(context.Context, RunProjectParam) (*job.Job, error)
}
