package gqlmodel

import (
	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
)

func ToWorkspace(t *accountsworkspace.Workspace) *Workspace {
	if t == nil {
		return nil
	}

	members := make([]*WorkspaceMember, 0, len(t.Members()))

	for _, member := range t.Members() {
		switch m := member.(type) {
		case accountsworkspace.UserMember:
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
		case accountsworkspace.IntegrationMember:
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

func FromRole(r Role) accountsworkspace.Role {
	switch r {
	case RoleReader:
		return accountsworkspace.RoleReader
	case RoleWriter:
		return accountsworkspace.RoleWriter
	case RoleMaintainer:
		return accountsworkspace.RoleMaintainer
	case RoleOwner:
		return accountsworkspace.RoleOwner
	}
	return accountsworkspace.Role("")
}
