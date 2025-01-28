package gqlmodel

import (
	"github.com/reearth/reearthx/account/accountdomain/workspace"
)

func ToWorkspace(t *workspace.Workspace) *Workspace {
	if t == nil {
		return nil
	}

	memberMap := t.Members().Users()
	members := make([]*WorkspaceMember, 0, len(memberMap))
	for u, _ := range memberMap {
		members = append(members, &WorkspaceMember{
			UserID: IDFrom(u),
		})
	}

	return &Workspace{
		ID:       IDFrom(t.ID()),
		Name:     t.Name(),
		Personal: t.IsPersonal(),
		Members:  members,
	}
}
