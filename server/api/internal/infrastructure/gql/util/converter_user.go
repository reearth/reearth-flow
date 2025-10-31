package util

import (
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/pkg/user"
	"github.com/samber/lo"
)

func ToMe(m gqlmodel.Me) (*user.User, error) {
	uid, err := user.IDFrom(string(m.ID))
	if err != nil {
		return nil, err
	}

	wid, err := user.WorkspaceIDFrom(string(m.MyWorkspaceID))
	if err != nil {
		return nil, err
	}

	return user.New().
		ID(uid).
		Name(string(m.Name)).
		Alias(string(m.Alias)).
		Email(string(m.Email)).
		Metadata(toUserMetadata(m.Metadata)).
		Host(lo.ToPtr(string(m.Host))).
		MyWorkspaceID(wid).
		Auths(toStringSlice(m.Auths)).
		Build()
}

func ToUser(u gqlmodel.User) (*user.User, error) {
	uid, err := user.IDFrom(string(u.ID))
	if err != nil {
		return nil, err
	}

	wid, err := user.WorkspaceIDFrom(string(u.Workspace))
	if err != nil {
		return nil, err
	}

	return user.New().
		ID(uid).
		Name(string(u.Name)).
		Email(string(u.Email)).
		Host(lo.ToPtr(string(u.Host))).
		MyWorkspaceID(wid).
		Auths(toStringSlice(u.Auths)).
		Metadata(toUserMetadata(u.Metadata)).
		Build()
}

func ToUserFromSimple(u gqlmodel.UserSimple) (*user.User, error) {
	uid, err := user.IDFrom(string(u.ID))
	if err != nil {
		return nil, err
	}

	return user.New().
		ID(uid).
		Name(string(u.Name)).
		Email(string(u.Email)).
		Host(lo.ToPtr(string(u.Host))).
		Build()
}

func ToUsers(gqlUsers []gqlmodel.User) (user.List, error) {
	users := make(user.List, 0, len(gqlUsers))
	for _, gu := range gqlUsers {
		u, err := ToUser(gu)
		if err != nil {
			return nil, err
		}
		users = append(users, u)
	}
	return users, nil
}
