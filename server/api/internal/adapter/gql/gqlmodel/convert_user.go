package gqlmodel

import (
	pkguser "github.com/reearth/reearth-flow/api/pkg/user"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/util"
	"github.com/samber/lo"
	"golang.org/x/text/language"
)

func ToUser(u *user.User) *User {
	if u == nil {
		return nil
	}

	return &User{
		ID:    IDFrom(u.ID()),
		Name:  u.Name(),
		Email: u.Email(),
		Host:  lo.EmptyableToPtr(u.Host()),
	}
}

// TODO: After migration, delete ToUser and rename ToUserFromFlow to ToUser.
func ToUserFromFlow(u pkguser.User) *User {
	return &User{
		ID:    IDFrom(u.ID()),
		Name:  u.Name(),
		Email: u.Email(),
		Host:  u.Host(),
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
	}
}

func ToMe(u *user.User) *Me {
	if u == nil {
		return nil
	}

	lang := language.English // Default language
	if u.Metadata() != nil {
		lang = u.Metadata().Lang()
	}

	return &Me{
		ID:            IDFrom(u.ID()),
		Name:          u.Name(),
		Email:         u.Email(),
		Lang:          lang,
		MyWorkspaceID: IDFrom(u.Workspace()),
		Auths: util.Map(u.Auths(), func(a user.Auth) string {
			return a.Provider
		}),
	}
}

// TODO: Keep using ToMe during the migration period.
// After migration, ToMe will be updated to handle FlowUser (from flow/pkg) and ToMeFromFlow will be removed.
func ToMeFromFlow(u *pkguser.User) *Me {
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
