package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/account/accountdomain"
	"golang.org/x/exp/slices"
)

type ProjectDocument struct {
	ID                string
	Archived          bool
	IsBasicAuthActive bool
	BasicAuthUsername string
	BasicAuthPassword string
	UpdatedAt         time.Time
	PublishedAt       time.Time
	Name              string
	Description       string
	Alias             string
	ImageURL          string
	PublicTitle       string
	PublicDescription string
	PublicImage       string
	PublicNoIndex     bool
	Workspace         string // DON'T CHANGE NAME'
	Workflow          string
	EnableGA          bool
	TrackingID        string
}

type ProjectConsumer = Consumer[*ProjectDocument, *project.Project]

func NewProjectConsumer(workspaces []accountdomain.WorkspaceID) *ProjectConsumer {
	return NewConsumer[*ProjectDocument, *project.Project](func(a *project.Project) bool {
		return workspaces == nil || slices.Contains(workspaces, a.Workspace())
	})
}

func NewProject(project *project.Project) (*ProjectDocument, string) {
	pid := project.ID().String()

	return &ProjectDocument{
		ID:                pid,
		Archived:          project.IsArchived(),
		IsBasicAuthActive: project.IsBasicAuthActive(),
		BasicAuthUsername: project.BasicAuthUsername(),
		BasicAuthPassword: project.BasicAuthPassword(),
		UpdatedAt:         project.UpdatedAt(),
		Name:              project.Name(),
		Description:       project.Description(),
		Workspace:         project.Workspace().String(),
		Workflow:          project.Workflow().String(),
	}, pid
}

func (d *ProjectDocument) Model() (*project.Project, error) {
	pid, err := id.ProjectIDFrom(d.ID)
	if err != nil {
		return nil, err
	}
	tid, err := accountdomain.WorkspaceIDFrom(d.Workspace)
	if err != nil {
		return nil, err
	}

	wid, _ := id.WorkflowIDFrom(d.Workflow)

	return project.New().
		ID(pid).
		IsArchived(d.Archived).
		IsBasicAuthActive(d.IsBasicAuthActive).
		BasicAuthUsername(d.BasicAuthUsername).
		BasicAuthPassword(d.BasicAuthPassword).
		UpdatedAt(d.UpdatedAt).
		Name(d.Name).
		Description(d.Description).
		Workspace(tid).
		Workflow(wid).
		Build()
}
