package gqlmodel

import (
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"

	"github.com/samber/lo"
)

func ToUser(u *accountsuser.User) *User {
	if u == nil {
		return nil
	}

	return &User{
		ID:       IDFrom(u.ID()),
		Name:     u.Name(),
		Email:    u.Email(),
		Host:     lo.EmptyableToPtr(u.Host()),
		Metadata: ToUserMetadata(u.Metadata()),
	}
}

func ToMe(u *accountsuser.User) *Me {
	if u == nil {
		return nil
	}

	auths := u.Auths()
	authStrs := make([]string, 0, len(auths))
	for _, a := range auths {
		authStrs = append(authStrs, a.String())
	}

	return &Me{
		ID:            IDFrom(u.ID()),
		Name:          u.Name(),
		Email:         u.Email(),
		Lang:          u.Metadata().Lang(),
		MyWorkspaceID: IDFrom(u.Workspace()),
		Auths:         authStrs,
	}
}

func ToUserMetadata(m *accountsuser.Metadata) *UserMetadata {
	if m == nil {
		return nil
	}

	return &UserMetadata{
		Description: lo.EmptyableToPtr(m.Description()),
		Website:     lo.EmptyableToPtr(m.Website()),
		Theme:       Theme(m.Theme()),
		PhotoURL:    lo.EmptyableToPtr(m.PhotoURL()),
		Lang:        m.Lang(),
	}
}
