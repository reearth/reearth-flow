package workspace

import "github.com/reearth/reearth-flow/api/internal/infrastructure/gql/gqlmodel"

type findByIDQuery struct {
	Workspace gqlmodel.Workspace `graphql:"findByID(id: $id)"`
}

type findByIDsQuery struct {
	Workspaces []gqlmodel.Workspace `graphql:"findByIDs(ids: $ids)"`
}

type findByUserQuery struct {
	Workspaces []gqlmodel.Workspace `graphql:"findByUser(userId: $userId)"`
}
