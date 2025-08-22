package gqlmodel

import (
	"github.com/hasura/go-graphql-client"
)

type WorkspaceMember struct {
	Typename       string `json:"__typename" graphql:"__typename"`
	UserMemberData struct {
		UserID graphql.ID     `json:"userId" graphql:"userId"`
		Role   graphql.String `json:"role" graphql:"role"`
		Host   graphql.String `json:"host" graphql:"host"`
		User   *User          `json:"user" graphql:"user"`
	} `graphql:"... on WorkspaceUserMember"`
	IntegrationMemberData struct {
		IntegrationID graphql.ID     `json:"integrationId" graphql:"integrationId"`
		Role          graphql.String `json:"role" graphql:"role"`
		Active        bool           `json:"active" graphql:"active"`
		InvitedByID   graphql.ID     `json:"invitedById" graphql:"invitedById"`
		InvitedBy     *User          `json:"invitedBy" graphql:"invitedBy"`
	} `graphql:"... on WorkspaceIntegrationMember"`
}
