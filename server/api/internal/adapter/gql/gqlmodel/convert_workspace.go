package gqlmodel

import (
	pkgworkspace "github.com/reearth/reearth-flow/api/pkg/workspace"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
)

func ToWorkspace(t *pkgworkspace.Workspace) *Workspace {
	if t == nil {
		return nil
	}

	members := make([]*WorkspaceMember, 0, len(t.Members()))

	for _, member := range t.Members() {
		switch m := member.(type) {
		case pkgworkspace.UserMember:
			workspaceMember := &WorkspaceMember{
				UserID: IDFrom(m.UserID),
				Role:   Role(m.Role),
			}
			if m.User != nil {
				workspaceMember.User = &User{
					ID:    IDFrom(m.User.ID),
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
		ID:       IDFrom(t.ID()),
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

func FromRole(r Role) pkgworkspace.Role {
	switch r {
	case RoleReader:
		return pkgworkspace.RoleReader
	case RoleWriter:
		return pkgworkspace.RoleWriter
	case RoleMaintainer:
		return pkgworkspace.RoleMaintainer
	case RoleOwner:
		return pkgworkspace.RoleOwner
	}
	return pkgworkspace.Role("")
}
