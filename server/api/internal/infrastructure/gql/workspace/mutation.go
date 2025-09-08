package workspace

import (
	"github.com/hasura/go-graphql-client"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gql/gqlmodel"
)

type createWorkspaceMutation struct {
	CreateWorkspace struct {
		Workspace gqlmodel.Workspace `graphql:"workspace"`
	} `graphql:"createWorkspace(input: $input)"`
}

type updateWorkspaceMutation struct {
	UpdateWorkspace struct {
		Workspace gqlmodel.Workspace `graphql:"workspace"`
	} `graphql:"updateWorkspace(input: $input)"`
}

type deleteWorkspaceMutation struct {
	DeleteWorkspace struct {
		WorkspaceID graphql.ID `graphql:"workspaceId"`
	} `graphql:"deleteWorkspace(input: $input)"`
}

type addUsersToWorkspaceMutation struct {
	AddUsersToWorkspace struct {
		Workspace gqlmodel.Workspace `graphql:"workspace"`
	} `graphql:"addUsersToWorkspace(input: $input)"`
}
