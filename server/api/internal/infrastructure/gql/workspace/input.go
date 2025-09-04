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
