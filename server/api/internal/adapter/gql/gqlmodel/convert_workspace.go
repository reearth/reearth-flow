package gqlmodel

import (
	"github.com/reearth/reearth-accounts/server/pkg/role"
	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
)

func ToWorkspace(t *accountsworkspace.Workspace) *Workspace {
	if t == nil {
		return nil
	}

	members := make([]*WorkspaceMember, 0, t.Members().Count())

	for userID, memberInfo := range t.Members().Users() {
		workspaceMember := &WorkspaceMember{
			UserID: IDFrom(userID),
			Role:   Role(memberInfo.Role),
			User: &User{
				ID: IDFrom(userID),
			},
		}

		members = append(members, workspaceMember)
	}

	return &Workspace{
		ID:       IDFrom(t.ID()),
		Name:     t.Name(),
		Personal: t.IsPersonal(),
		Members:  members,
	}
}

func FromRole(r Role) role.RoleType {
	switch r {
	case RoleOwner:
		return role.RoleOwner
	case RoleMaintainer:
		return role.RoleMaintainer
	case RoleWriter:
		return role.RoleWriter
	case RoleReader:
		return role.RoleReader
	default:
		return role.RoleType("")
	}
}
