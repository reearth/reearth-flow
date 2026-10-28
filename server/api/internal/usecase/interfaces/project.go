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

// ProjectSortField orders a project search.
type ProjectSortField string

const (
	ProjectSortUpdatedAt ProjectSortField = "updatedAt"
	ProjectSortName      ProjectSortField = "name"
	ProjectSortStarCount ProjectSortField = "starCount"
)

// ProjectSearchParam filters and pages a project search. A nil Workspace
// searches every workspace, Topics matches projects that have all of them, and
// an empty SortBy sorts by update time. Page starts at 1.
type ProjectSearchParam struct {
	Workspace       *accountsid.WorkspaceID
	Visibility      *project.Visibility
	Keyword         string
	SortBy          ProjectSortField
	Topics          []string
	Page            int
	PageSize        int
	SortAscending   bool
	IncludeArchived bool
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
	FindByWorkspace(context.Context, accountsid.WorkspaceID, *PaginationParam, *string, *bool, accountsid.UserIDList) ([]*project.Project, *PageBasedInfo, error)
	// FindViewable returns the project if the caller may see it: a workspace
	// member always, anyone else only while it is public and not archived. A
	// hidden project returns rerror.ErrNotFound, the same as a missing one.
	FindViewable(context.Context, id.ProjectID) (*project.Project, error)
	// FindViewableByAlias is FindViewable for the project with the alias in the workspace.
	FindViewableByAlias(ctx context.Context, wid accountsid.WorkspaceID, alias string) (*project.Project, error)
	// Search lists the projects the caller may see. In a workspace the caller
	// is a member of, the search runs as given; anywhere else, including with
	// no Workspace, it returns only public projects that are not archived.
	Search(context.Context, ProjectSearchParam) ([]*project.Project, *PageBasedInfo, error)
	// CanCreatePrivate reports whether the workspace's plan allows private projects.
	CanCreatePrivate(ctx context.Context, wid accountsid.WorkspaceID) (bool, error)
	// CanScheduleHourly reports whether the workspace's plan allows triggers that run every hour.
	CanScheduleHourly(ctx context.Context, wid accountsid.WorkspaceID) (bool, error)
	// CheckAlias reports whether alias is free in the workspace. Pass the
	// project's own ID when editing it, so its current alias counts as free.
	CheckAlias(ctx context.Context, wid accountsid.WorkspaceID, alias string, pid *id.ProjectID) (bool, error)
	Create(context.Context, CreateProjectParam) (*project.Project, error)
	Update(context.Context, UpdateProjectParam) (*project.Project, error)
	Delete(context.Context, id.ProjectID) error
	Run(context.Context, RunProjectParam) (*job.Job, error)
	PreviewSchema(context.Context, PreviewSchemaParam) (*job.Job, error)
	// SetStar stars or unstars the project for the current user. Workspace
	// members may star any project they can read; anyone may star a project
	// they can view because it is public.
	SetStar(ctx context.Context, pid id.ProjectID, starred bool) (*project.Project, error)
}
