package mongodoc

import (
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"golang.org/x/exp/slices"
)

type ProjectDocument struct {
	PublishedAt time.Time
	UpdatedAt   time.Time

	// Core Identity
	ID        string
	Alias     string
	Name      string
	Workflow  string
	Workspace string

	// Authorship: user IDs, absent on projects saved before they were recorded
	CreatedBy *string
	UpdatedBy *string

	// Authentication
	BasicAuthPassword string
	BasicAuthUsername string

	// Content
	Description string
	ImageURL    string
	Readme      string
	License     string
	Topics      []string
	Visibility  string

	// Written only by the star updates; omitted from saves so an edit made from
	// a stale read cannot drop a star added in the meantime.
	StarredBy []string `bson:",omitempty"`

	// Public Visibility Configuration
	PublicDescription string
	PublicImage       string
	PublicTitle       string
	SharedToken       string

	TrackingID        string
	IsBasicAuthActive bool

	// Metadata
	Archived      bool
	PublicNoIndex bool
	IsLocked      bool

	// Analytics
	EnableGA bool
}

type ProjectConsumer = Consumer[*ProjectDocument, *project.Project]

func NewProjectConsumer(workspaces []accountsid.WorkspaceID) *ProjectConsumer {
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
		CreatedBy:         userIDString(project.CreatedBy()),
		UpdatedBy:         userIDString(project.UpdatedBy()),
		Alias:             project.Alias(),
		Archived:          project.IsArchived(),
		BasicAuthPassword: project.BasicAuthPassword(),
		BasicAuthUsername: project.BasicAuthUsername(),
		Description:       project.Description(),
		IsBasicAuthActive: project.IsBasicAuthActive(),
		IsLocked:          project.IsLocked(),
		License:           project.License(),
		Name:              project.Name(),
		Readme:            project.Readme(),
		SharedToken:       sharedToken,
		StarredBy:         project.StarredBy(),
		Topics:            project.Topics(),
		UpdatedAt:         project.UpdatedAt(),
		Visibility:        string(project.Visibility()),
		Workflow:          project.Workflow().String(),
		Workspace:         project.Workspace().String(),
	}, pid
}

func (d *ProjectDocument) Model() (*project.Project, error) {
	pid, err := id.ProjectIDFrom(d.ID)
	if err != nil {
		return nil, err
	}

	tid, err := accountsid.WorkspaceIDFrom(d.Workspace)
	if err != nil {
		return nil, err
	}

	wid, _ := id.WorkflowIDFrom(d.Workflow)

	var sharedToken *string
	if d.SharedToken != "" {
		sharedToken = &d.SharedToken
	}

	createdBy, err := userIDFrom(d.CreatedBy)
	if err != nil {
		return nil, err
	}
	updatedBy, err := userIDFrom(d.UpdatedBy)
	if err != nil {
		return nil, err
	}

	return project.New().
		ID(pid).
		CreatedBy(createdBy).
		UpdatedBy(updatedBy).
		Alias(d.Alias).
		BasicAuthPassword(d.BasicAuthPassword).
		BasicAuthUsername(d.BasicAuthUsername).
		Description(d.Description).
		IsArchived(d.Archived).
		IsBasicAuthActive(d.IsBasicAuthActive).
		IsLocked(d.IsLocked).
		License(d.License).
		Name(d.Name).
		Readme(d.Readme).
		SharedToken(sharedToken).
		StarredBy(d.StarredBy).
		Topics(d.Topics).
		UpdatedAt(d.UpdatedAt).
		Visibility(project.Visibility(d.Visibility)).
		Workflow(wid).
		Workspace(tid).
		Build()
}

func userIDString(uid *accountsid.UserID) *string {
	if uid == nil {
		return nil
	}
	s := uid.String()
	return &s
}

func userIDFrom(s *string) (*accountsid.UserID, error) {
	if s == nil || *s == "" {
		return nil, nil
	}
	uid, err := accountsid.UserIDFrom(*s)
	if err != nil {
		return nil, err
	}
	return &uid, nil
}
