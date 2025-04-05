package mongodoc

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/projectAccess"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
)

type UserRoleDocument struct {
	UserID string
	Role   string
}

type ProjectAccessDocument struct {
	ID        string
	Project   string
	IsPublic  bool
	Token     string
	UserRoles []UserRoleDocument
}

type ProjectAccessConsumer = Consumer[*ProjectAccessDocument, *projectAccess.ProjectAccess]

func NewProjectAccessConsumer() *ProjectAccessConsumer {
	return NewConsumer[*ProjectAccessDocument, *projectAccess.ProjectAccess](func(a *projectAccess.ProjectAccess) bool {
		return true
	})
}

func NewProjectAccess(projectAccess *projectAccess.ProjectAccess) (*ProjectAccessDocument, string) {
	paid := projectAccess.ID().String()

	userRoles := make([]UserRoleDocument, 0, len(projectAccess.UserRoles()))
	for _, ur := range projectAccess.UserRoles() {
		userRoles = append(userRoles, UserRoleDocument{
			UserID: ur.UserID().String(),
			Role:   string(ur.Role()),
		})
	}

	return &ProjectAccessDocument{
		ID:        paid,
		Project:   projectAccess.Project().String(),
		IsPublic:  projectAccess.IsPublic(),
		Token:     projectAccess.Token(),
		UserRoles: userRoles,
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

	userRoles := projectAccess.NewUserRoleList()
	for _, ur := range d.UserRoles {
		uid, err := user.IDFrom(ur.UserID)
		if err != nil {
			return nil, err
		}

		role := workspace.Role(ur.Role)

		err = userRoles.Add(uid, role)
		if err != nil {
			return nil, err
		}
	}

	return projectAccess.New().
		ID(paid).
		Project(pid).
		IsPublic(d.IsPublic).
		Token(d.Token).
		UserRoles(userRoles).
		Build()
}
