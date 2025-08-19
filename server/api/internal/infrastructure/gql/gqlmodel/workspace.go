package gqlmodel

import "github.com/hasura/go-graphql-client"

type Workspace struct {
	ID       graphql.ID        `json:"id" graphql:"id"`
	Name     graphql.String    `json:"name" graphql:"name"`
	Alias    graphql.String    `json:"alias" graphql:"alias"`
	Metadata WorkspaceMetadata `json:"metadata" graphql:"metadata"`
	Personal bool              `json:"personal" graphql:"personal"`
	Members  []WorkspaceMember `json:"members" graphql:"members"`
}
