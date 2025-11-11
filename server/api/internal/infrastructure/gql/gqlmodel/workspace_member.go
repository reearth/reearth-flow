package gqlmodel

import (
	"github.com/hasura/go-graphql-client"
)

type WorkspaceMember struct {
	IntegrationMemberData struct {
		InvitedBy     *User          `json:"invitedBy" graphql:"invitedBy"`
		IntegrationID graphql.ID     `json:"integrationId" graphql:"integrationId"`
		Role          graphql.String `json:"role" graphql:"role"`
		InvitedByID   graphql.ID     `json:"invitedById" graphql:"invitedById"`
		Active        bool           `json:"active" graphql:"active"`
	} `graphql:"... on WorkspaceIntegrationMember"`
	UserMemberData struct {
		User   *User          `json:"user" graphql:"user"`
		UserID graphql.ID     `json:"userId" graphql:"userId"`
		Role   graphql.String `json:"role" graphql:"role"`
		Host   graphql.String `json:"host" graphql:"host"`
	} `graphql:"... on WorkspaceUserMember"`
	Typename string `json:"__typename" graphql:"__typename"`
}
