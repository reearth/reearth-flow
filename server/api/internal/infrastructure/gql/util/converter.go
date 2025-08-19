package util

import (
	"github.com/hasura/go-graphql-client"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/pkg/user"
	"github.com/reearth/reearth-flow/api/pkg/workspace"
	"golang.org/x/text/language"
)

func ToWorkspace(w gqlmodel.Workspace) *workspace.Workspace {
	return workspace.New().
		ID(string(w.ID)).
		Name(string(w.Name)).
		Alias(string(w.Alias)).
		Metadata(ToWorkspaceMetadata(w.Metadata)).
		Personal(w.Personal).
		Members(ToWorkspaceMembers(w.Members)).
		MustBuild()
}

func ToWorkspaces(gqlWorkspaces []gqlmodel.Workspace) workspace.WorkspaceList {
	workspaces := make(workspace.WorkspaceList, 0, len(gqlWorkspaces))
	for _, w := range gqlWorkspaces {
		if ws := ToWorkspace(w); ws != nil {
			workspaces = append(workspaces, *ws)
		}
	}
	return workspaces
}

func ToUserMetadata(m gqlmodel.UserMetadata) user.Metadata {
	return user.NewMetadata().
		Description(string(m.Description)).
		Lang(language.Make(string(m.Lang))).
		PhotoURL(string(m.PhotoURL)).
		Theme(string(m.Theme)).
		Website(string(m.Website)).
		MustBuild()
}

func ToWorkspaceMetadata(m gqlmodel.WorkspaceMetadata) workspace.Metadata {
	return workspace.NewMetadata().
		Description(string(m.Description)).
		Website(string(m.Website)).
		Location(string(m.Location)).
		BillingEmail(string(m.BillingEmail)).
		PhotoURL(string(m.PhotoURL)).
		MustBuild()
}

func ToWorkspaceMembers(gqlMembers []gqlmodel.WorkspaceMember) []workspace.Member {
	var members []workspace.Member

	for _, gqlMember := range gqlMembers {
		switch gqlMember.Typename {
		case "WorkspaceUserMember":
			if gqlMember.UserMemberData.UserID != "" {
				userMember := workspace.UserMember{
					UserID: workspace.UserID(gqlMember.UserMemberData.UserID),
					Role:   workspace.Role(gqlMember.UserMemberData.Role),
				}
				if gqlMember.UserMemberData.Host != "" {
					hostStr := string(gqlMember.UserMemberData.Host)
					userMember.Host = &hostStr
				}
				if gqlMember.UserMemberData.User != nil {
					userMember.User = &workspace.User{
						ID:    workspace.UserID(gqlMember.UserMemberData.User.ID),
						Name:  string(gqlMember.UserMemberData.User.Name),
						Email: string(gqlMember.UserMemberData.User.Email),
					}
				}
				members = append(members, userMember)
			}

		case "WorkspaceIntegrationMember":
			if gqlMember.IntegrationMemberData.IntegrationID != "" {
				integrationMember := workspace.IntegrationMember{
					IntegrationID: workspace.IntegrationID(gqlMember.IntegrationMemberData.IntegrationID),
					Role:          workspace.Role(gqlMember.IntegrationMemberData.Role),
					Active:        gqlMember.IntegrationMemberData.Active,
					InvitedByID:   workspace.UserID(gqlMember.IntegrationMemberData.InvitedByID),
				}
				if gqlMember.IntegrationMemberData.InvitedBy != nil {
					integrationMember.InvitedBy = &workspace.User{
						ID:    workspace.UserID(gqlMember.IntegrationMemberData.InvitedBy.ID),
						Name:  string(gqlMember.IntegrationMemberData.InvitedBy.Name),
						Email: string(gqlMember.IntegrationMemberData.InvitedBy.Email),
					}
				}
				members = append(members, integrationMember)
			}
		}
	}

	return members
}

func FromPtrToPtr(s *graphql.String) *string {
	if s == nil {
		return nil
	}
	str := string(*s)
	return &str
}

func ToStringSlice(gqlSlice []graphql.String) []string {
	res := make([]string, len(gqlSlice))
	for i, v := range gqlSlice {
		res[i] = string(v)
	}
	return res
}
