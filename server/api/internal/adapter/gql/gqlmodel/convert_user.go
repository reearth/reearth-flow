package gqlmodel

import (
	"github.com/reearth/reearth-flow/api/pkg/user"
	"github.com/samber/lo"
)

func ToUser(u *user.User) *User {
	if u == nil {
		return nil
	}

	return &User{
		ID:       IDFrom(u.ID()),
		Name:     u.Name(),
		Email:    u.Email(),
		Host:     u.Host(),
		Metadata: ToUserMetadata(u.Metadata()),
	}
}

func ToMe(u *user.User) *Me {
	if u == nil {
		return nil
	}

	return &Me{
		ID:            IDFrom(u.ID()),
		Name:          u.Name(),
		Email:         u.Email(),
		Lang:          u.Metadata().Lang(),
		MyWorkspaceID: IDFrom(u.MyWorkspaceID()),
		Auths:         u.Auths(),
	}
}

func ToUserMetadata(m user.Metadata) *UserMetadata {
	return &UserMetadata{
		Description: lo.EmptyableToPtr(m.Description()),
		Website:     lo.EmptyableToPtr(m.Website()),
		Theme:       Theme(m.Theme()),
		PhotoURL:    lo.EmptyableToPtr(m.PhotoURL()),
		Lang:        m.Lang(),
	}
}
