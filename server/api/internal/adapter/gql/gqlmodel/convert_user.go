package gqlmodel

import (
	pkguser "github.com/reearth/reearth-flow/api/pkg/user"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/util"
	"github.com/samber/lo"
	"golang.org/x/text/language"
)

func ToUser(u *pkguser.User) *User {
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

func ToUsersFromSimple(users user.SimpleList) []*User {
	return util.Map(users, ToUserFromSimple)
}

func ToUserFromSimple(u *user.Simple) *User {
	if u == nil {
		return nil
	}

	return &User{
		ID:    IDFrom(u.ID),
		Name:  u.Name,
		Email: u.Email,
		Metadata: &UserMetadata{
			Lang: language.English,
		},
	}
}

func ToMe(u *pkguser.User) *Me {
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

func ToUserMetadata(m pkguser.Metadata) *UserMetadata {
	return &UserMetadata{
		Description: lo.EmptyableToPtr(m.Description()),
		Website:     lo.EmptyableToPtr(m.Website()),
		Theme:       Theme(m.Theme()),
		PhotoURL:    lo.EmptyableToPtr(m.PhotoURL()),
		Lang:        m.Lang(),
	}
}

func ToUserMetadataFromAccount(m *user.Metadata) *UserMetadata {
	if m == nil {
		return &UserMetadata{
			Lang: language.English,
		}
	}
	return &UserMetadata{
		Description: lo.EmptyableToPtr(m.Description()),
		Website:     lo.EmptyableToPtr(m.Website()),
		PhotoURL:    lo.EmptyableToPtr(m.PhotoURL()),
		Theme:       Theme(m.Theme()),
		Lang:        m.Lang(),
	}
}
