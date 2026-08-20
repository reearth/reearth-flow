package interfaces

import (
	"context"
	"errors"

	"github.com/google/uuid"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/project"
)

type CreateProjectParam struct {
	Name        *string
	Description *string
	Archived    *bool
	WorkspaceID accountsid.WorkspaceID
}

type UpdateProjectParam struct {
	Name              *string
	Description       *string
	Archived          *bool
	IsBasicAuthActive *bool
	IsLocked          *bool
	BasicAuthUsername *string
	BasicAuthPassword *string
	ID                id.ProjectID
}

type RunProjectParam struct {
	Workflow      *file.File
	ProjectID     id.ProjectID
	PreviousJobID *id.JobID
	StartNodeID   *uuid.UUID
	Parameters    []*parameter.Parameter
}

type PreviewSchemaParam struct {
	Workflow   *file.File
	SampleSize *int
	Parameters []*parameter.Parameter
	ProjectID  id.ProjectID
}

var (
	ErrProjectAliasIsNotSet    error = errors.New("project alias is not set")
	ErrProjectAliasAlreadyUsed error = errors.New("project alias is already used by another project")
	ErrWorkflowFileRequired    error = errors.New("workflow file is required")
)

type Project interface {
	Fetch(context.Context, []id.ProjectID) ([]*project.Project, error)
	FindByWorkspace(context.Context, accountsid.WorkspaceID, *PaginationParam, *string, *bool) ([]*project.Project, *PageBasedInfo, error)
	Create(context.Context, CreateProjectParam) (*project.Project, error)
	Update(context.Context, UpdateProjectParam) (*project.Project, error)
	Delete(context.Context, id.ProjectID) error
	Run(context.Context, RunProjectParam) (*job.Job, error)
	PreviewSchema(context.Context, PreviewSchemaParam) (*job.Job, error)
}
