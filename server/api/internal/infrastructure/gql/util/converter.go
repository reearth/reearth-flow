package util

import (
	"errors"
	"fmt"

	"github.com/hasura/go-graphql-client"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/user"
	"github.com/reearth/reearth-flow/api/pkg/workspace"
	"github.com/samber/lo"
	"golang.org/x/text/language"
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

	workspaces, err := ToWorkspaces(m.Workspaces)
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
		Workspaces(workspaces).
		Build()
}

func toUser(u gqlmodel.User) (*user.User, error) {
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

func ToUsers(gqlUsers []gqlmodel.User) (user.List, error) {
	users := make(user.List, 0, len(gqlUsers))
	for _, gu := range gqlUsers {
		u, err := toUser(gu)
		if err != nil {
			return nil, err
		}
		users = append(users, *u)
	}
	return users, nil
}

func toWorkspace(w gqlmodel.Workspace) (*workspace.Workspace, error) {
	wid, err := workspace.IDFrom(string(w.ID))
	if err != nil {
		return nil, err
	}

	members, err := toWorkspaceMembers(w.Members)
	if err != nil {
		return nil, err
	}

	return workspace.New().
		ID(wid).
		Name(string(w.Name)).
		Alias(string(w.Alias)).
		Metadata(toWorkspaceMetadata(w.Metadata)).
		Personal(w.Personal).
		Members(members).
		Build()
}

func ToWorkspaces(gqlWorkspaces []gqlmodel.Workspace) (workspace.List, error) {
	workspaces := make(workspace.List, 0, len(gqlWorkspaces))
	for _, w := range gqlWorkspaces {
		ws, err := toWorkspace(w)
		if err != nil {
			return nil, err
		}
		workspaces = append(workspaces, *ws)
	}
	return workspaces, nil
}

func toUserMetadata(m gqlmodel.UserMetadata) user.Metadata {
	return user.NewMetadata().
		Description(string(m.Description)).
		Lang(language.Make(string(m.Lang))).
		PhotoURL(string(m.PhotoURL)).
		Theme(string(m.Theme)).
		Website(string(m.Website)).
		MustBuild()
}

func toWorkspaceMetadata(m gqlmodel.WorkspaceMetadata) workspace.Metadata {
	return workspace.NewMetadata().
		Description(string(m.Description)).
		Website(string(m.Website)).
		Location(string(m.Location)).
		BillingEmail(string(m.BillingEmail)).
		PhotoURL(string(m.PhotoURL)).
		MustBuild()
}

func toWorkspaceMembers(gqlMembers []gqlmodel.WorkspaceMember) ([]workspace.Member, error) {
	var members []workspace.Member

	for _, gqlMember := range gqlMembers {
		switch gqlMember.Typename {
		case "WorkspaceUserMember":
			member, err := toUserMember(gqlMember)
			if err != nil {
				return nil, err
			}
			members = append(members, member)

		case "WorkspaceIntegrationMember":
			member, err := toIntegrationMember(gqlMember)
			if err != nil {
				return nil, err
			}
			members = append(members, member)

		default:
			return nil, fmt.Errorf("unknown workspace member type: %s", gqlMember.Typename)
		}
	}

	return members, nil
}

func toUserMember(gql gqlmodel.WorkspaceMember) (workspace.UserMember, error) {
	if gql.UserMemberData.UserID == "" {
		return workspace.UserMember{}, errors.New("missing user ID")
	}
	uid, err := workspace.UserIDFrom(string(gql.UserMemberData.UserID))
	if err != nil {
		return workspace.UserMember{}, err
	}

	member := workspace.UserMember{
		UserID: uid,
		Role:   workspace.Role(gql.UserMemberData.Role),
	}
	if gql.UserMemberData.Host != "" {
		hostStr := string(gql.UserMemberData.Host)
		member.Host = &hostStr
	}
	if gql.UserMemberData.User != nil {
		id, err := workspace.UserIDFrom(string(gql.UserMemberData.User.ID))
		if err != nil {
			return workspace.UserMember{}, err
		}
		member.User = &workspace.User{
			ID:    id,
			Name:  string(gql.UserMemberData.User.Name),
			Email: string(gql.UserMemberData.User.Email),
		}
	}
	return member, nil
}

func toIntegrationMember(gql gqlmodel.WorkspaceMember) (workspace.IntegrationMember, error) {
	if gql.IntegrationMemberData.IntegrationID == "" {
		return workspace.IntegrationMember{}, errors.New("missing integration ID")
	}

	itid, err := id.IntegrationIDFrom(string(gql.IntegrationMemberData.IntegrationID))
	if err != nil {
		return workspace.IntegrationMember{}, err
	}

	ivid, err := workspace.UserIDFrom(string(gql.IntegrationMemberData.InvitedByID))
	if err != nil {
		return workspace.IntegrationMember{}, err
	}

	member := workspace.IntegrationMember{
		IntegrationID: itid,
		Role:          workspace.Role(gql.IntegrationMemberData.Role),
		Active:        gql.IntegrationMemberData.Active,
		InvitedByID:   ivid,
	}

	if gql.IntegrationMemberData.InvitedBy != nil {
		id, err := workspace.UserIDFrom(string(gql.IntegrationMemberData.InvitedBy.ID))
		if err != nil {
			return workspace.IntegrationMember{}, err
		}
		member.InvitedBy = &workspace.User{
			ID:    id,
			Name:  string(gql.IntegrationMemberData.InvitedBy.Name),
			Email: string(gql.IntegrationMemberData.InvitedBy.Email),
		}
	}
	return member, nil
}

func FromPtrToPtr(s *graphql.String) *string {
	if s == nil {
		return nil
	}
	str := string(*s)
	return &str
}

func toStringSlice(gqlSlice []graphql.String) []string {
	res := make([]string, len(gqlSlice))
	for i, v := range gqlSlice {
		res[i] = string(v)
	}
	return res
}
