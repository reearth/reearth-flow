package workspace

import (
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gql/gqlmodel"
)

type createWorkspaceMutation struct {
	CreateWorkspace struct {
		Workspace gqlmodel.Workspace `graphql:"workspace"`
	} `graphql:"createWorkspace(input: $input)"`
}
