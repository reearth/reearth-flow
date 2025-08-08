package user

import "github.com/reearth/reearth-flow/api/internal/infrastructure/gql/gqlmodel"

type findMeQuery struct {
	Me gqlmodel.Me `graphql:"me"`
}
