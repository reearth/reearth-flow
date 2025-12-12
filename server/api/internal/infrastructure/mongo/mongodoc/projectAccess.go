package mongodoc

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/projectAccess"
)

type ProjectAccessDocument struct {
	ID       string
	Project  string
	Token    string
	IsPublic bool
}

type ProjectAccessConsumer = Consumer[*ProjectAccessDocument, *projectAccess.ProjectAccess]

func NewProjectAccessConsumer() *ProjectAccessConsumer {
	return NewConsumer[*ProjectAccessDocument, *projectAccess.ProjectAccess](func(a *projectAccess.ProjectAccess) bool {
		return true
	})
}

func NewProjectAccess(projectAccess *projectAccess.ProjectAccess) (*ProjectAccessDocument, string) {
	paid := projectAccess.ID().String()

	return &ProjectAccessDocument{
		ID:       paid,
		Project:  projectAccess.Project().String(),
		IsPublic: projectAccess.IsPublic(),
		Token:    projectAccess.Token(),
	}, paid
}

func (d *ProjectAccessDocument) Model() (*projectAccess.ProjectAccess, error) {
	paid, err := id.ProjectAccessIDFrom(d.ID)
	if err != nil {
		return nil, err
	}
	pid, err := id.ProjectIDFrom(d.Project)
	if err != nil {
		return nil, err
	}

	return projectAccess.New().
		ID(paid).
		Project(pid).
		IsPublic(d.IsPublic).
		Token(d.Token).
		Build()
}
