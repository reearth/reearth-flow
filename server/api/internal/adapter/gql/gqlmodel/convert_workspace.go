package gqlmodel

import (
	pkgworkspace "github.com/reearth/reearth-flow/api/pkg/workspace"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
)

func ToWorkspace(t *workspace.Workspace) *Workspace {
	if t == nil {
		return nil
	}

	memberMap := t.Members().Users()
	members := make([]*WorkspaceMember, 0, len(memberMap))
	for u, r := range memberMap {
		members = append(members, &WorkspaceMember{
			UserID: IDFrom(u),
			Role:   ToRole(r.Role),
		})
	}

	return &Workspace{
		ID:       IDFrom(t.ID()),
		Name:     t.Name(),
		Personal: t.IsPersonal(),
		Members:  members,
	}
}

// TODO: After migration, delete ToWorkspace and rename ToWorkspaceFromFlow to ToWorkspace.
func ToWorkspaceFromFlow(t pkgworkspace.Workspace) *Workspace {
	members := make([]*WorkspaceMember, 0, len(t.Members()))

	for _, member := range t.Members() {
		switch m := member.(type) {
		case pkgworkspace.UserMember:
			workspaceMember := &WorkspaceMember{
				UserID: ID(m.UserID),
				Role:   Role(m.Role),
			}
			if m.User != nil {
				workspaceMember.User = &User{
					ID:    ID(m.User.ID),
					Name:  m.User.Name,
					Email: m.User.Email,
					Host:  m.Host,
				}
			}
			members = append(members, workspaceMember)
		case pkgworkspace.IntegrationMember:
			// For IntegrationMember, the current WorkspaceMember structure does not support it.
			continue
		}
	}

	return &Workspace{
		ID:       ID(t.ID()),
		Name:     t.Name(),
		Personal: t.Personal(),
		Members:  members,
	}
}

func ToRole(r workspace.Role) Role {
	switch r {
	case workspace.RoleReader:
		return RoleReader
	case workspace.RoleWriter:
		return RoleWriter
	case workspace.RoleMaintainer:
		return RoleMaintainer
	case workspace.RoleOwner:
		return RoleOwner
	}
	return Role("")
}

func FromRole(r Role) workspace.Role {
	switch r {
	case RoleReader:
		return workspace.RoleReader
	case RoleWriter:
		return workspace.RoleWriter
	case RoleMaintainer:
		return workspace.RoleMaintainer
	case RoleOwner:
		return workspace.RoleOwner
	}
	return workspace.Role("")
}
