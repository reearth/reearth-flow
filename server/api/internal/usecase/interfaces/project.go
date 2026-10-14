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

// In CreateProjectParam and UpdateProjectParam a nil field is left as it is; a
// non-nil empty Topics clears the topics.
type CreateProjectParam struct {
	Name        *string
	Description *string
	Archived    *bool
	Alias       *string
	Visibility  *project.Visibility
	Readme      *string
	License     *string
	Topics      []string
	WorkspaceID accountsid.WorkspaceID
}

type UpdateProjectParam struct {
	Name              *string
	Description       *string
	Archived          *bool
	Alias             *string
	Visibility        *project.Visibility
	Readme            *string
	License           *string
	Topics            []string
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
	ErrProjectAliasAlreadyUsed  error = errors.New("project alias is already used by another project")
	ErrPrivateProjectNotAllowed error = errors.New("this workspace's plan does not allow private projects")
	ErrWorkflowFileRequired     error = errors.New("workflow file is required")
)

type Project interface {
	Fetch(context.Context, []id.ProjectID) ([]*project.Project, error)
	FindByWorkspace(context.Context, accountsid.WorkspaceID, *PaginationParam, *string, *bool) ([]*project.Project, *PageBasedInfo, error)
	// CanCreatePrivate reports whether the workspace's plan allows private projects.
	CanCreatePrivate(ctx context.Context, wid accountsid.WorkspaceID) (bool, error)
	// CheckAlias reports whether alias is free in the workspace. Pass the
	// project's own ID when editing it, so its current alias counts as free.
	CheckAlias(ctx context.Context, wid accountsid.WorkspaceID, alias string, pid *id.ProjectID) (bool, error)
	Create(context.Context, CreateProjectParam) (*project.Project, error)
	Update(context.Context, UpdateProjectParam) (*project.Project, error)
	Delete(context.Context, id.ProjectID) error
	Run(context.Context, RunProjectParam) (*job.Job, error)
	PreviewSchema(context.Context, PreviewSchemaParam) (*job.Job, error)
	// SetStar stars or unstars the project for the current user.
	SetStar(ctx context.Context, pid id.ProjectID, starred bool) (*project.Project, error)
}
