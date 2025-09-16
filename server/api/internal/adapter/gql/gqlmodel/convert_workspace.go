package gqlmodel

import (
	"github.com/reearth/reearth-flow/api/pkg/workspace"
	reearthxworkspace "github.com/reearth/reearthx/account/accountdomain/workspace"
)

func ToWorkspace(t *workspace.Workspace) *Workspace {
	if t == nil {
		return nil
	}

	members := make([]*WorkspaceMember, 0, len(t.Members()))

	for _, member := range t.Members() {
		switch m := member.(type) {
		case workspace.UserMember:
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
		case workspace.IntegrationMember:
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

// TODO: After migration, rename this function to ToRole
func ToRoleFromReearthx(r reearthxworkspace.Role) Role {
	switch r {
	case reearthxworkspace.RoleReader:
		return RoleReader
	case reearthxworkspace.RoleWriter:
		return RoleWriter
	case reearthxworkspace.RoleMaintainer:
		return RoleMaintainer
	case reearthxworkspace.RoleOwner:
		return RoleOwner
	}
	return Role("")
}

// TODO: After migration, remove this function
func FromRoleToReearthx(r Role) reearthxworkspace.Role {
	switch r {
	case RoleReader:
		return reearthxworkspace.RoleReader
	case RoleWriter:
		return reearthxworkspace.RoleWriter
	case RoleMaintainer:
		return reearthxworkspace.RoleMaintainer
	case RoleOwner:
		return reearthxworkspace.RoleOwner
	}
	return reearthxworkspace.Role("")
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
