package workspace

import (
	"github.com/hasura/go-graphql-client"
)

type CreateWorkspaceInput struct {
	Name graphql.String `json:"name"`
}

type UpdateWorkspaceInput struct {
	WorkspaceID graphql.ID     `json:"workspaceId"`
	Name        graphql.String `json:"name"`
}

type DeleteWorkspaceInput struct {
	WorkspaceID graphql.ID `json:"workspaceId"`
}

type AddUsersToWorkspaceInput struct {
	WorkspaceID graphql.ID    `json:"workspaceId"`
	Users       []MemberInput `json:"users"`
}

type MemberInput struct {
	UserID graphql.ID     `json:"userId"`
	Role   graphql.String `json:"role"`
}
