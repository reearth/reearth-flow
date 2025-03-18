package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/account/accountdomain"
	"golang.org/x/exp/slices"
)

type ProjectDocument struct {
	// Core Identity
	ID        string
	Alias     string
	Name      string
	Workflow  string
	Workspace string

	// Authentication
	BasicAuthPassword string
	BasicAuthUsername string
	IsBasicAuthActive bool

	// Content
	Description string
	ImageURL    string

	// Metadata
	Archived    bool
	PublishedAt time.Time
	UpdatedAt   time.Time

	// Public Visibility Configuration
	PublicDescription string
	PublicImage       string
	PublicNoIndex     bool
	PublicTitle       string
	SharedToken       string

	// Analytics
	EnableGA   bool
	TrackingID string
}

type ProjectConsumer = Consumer[*ProjectDocument, *project.Project]

func NewProjectConsumer(workspaces []accountdomain.WorkspaceID) *ProjectConsumer {
	return NewConsumer[*ProjectDocument](func(a *project.Project) bool {
		return workspaces == nil || slices.Contains(workspaces, a.Workspace())
	})
}

func NewProject(project *project.Project) (*ProjectDocument, string) {
	pid := project.ID().String()

	var sharedToken string
	if project.SharedToken() != nil {
		sharedToken = *project.SharedToken()
	}

	return &ProjectDocument{
		ID:                pid,
		Archived:          project.IsArchived(),
		BasicAuthPassword: project.BasicAuthPassword(),
		BasicAuthUsername: project.BasicAuthUsername(),
		Description:       project.Description(),
		IsBasicAuthActive: project.IsBasicAuthActive(),
		Name:              project.Name(),
		SharedToken:       sharedToken,
		UpdatedAt:         project.UpdatedAt(),
		Workflow:          project.Workflow().String(),
		Workspace:         project.Workspace().String(),
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

	var sharedToken *string
	if d.SharedToken != "" {
		sharedToken = &d.SharedToken
	}

	return project.New().
		ID(pid).
		BasicAuthPassword(d.BasicAuthPassword).
		BasicAuthUsername(d.BasicAuthUsername).
		Description(d.Description).
		IsArchived(d.Archived).
		IsBasicAuthActive(d.IsBasicAuthActive).
		Name(d.Name).
		SharedToken(sharedToken).
		UpdatedAt(d.UpdatedAt).
		Workflow(wid).
		Workspace(tid).
		Build()
}
