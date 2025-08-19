package workspace

import "github.com/reearth/reearth-flow/api/internal/infrastructure/gql/gqlmodel"

type findByIDsQuery struct {
	Workspaces []gqlmodel.Workspace `graphql:"findByIDs(ids: $ids)"`
}
