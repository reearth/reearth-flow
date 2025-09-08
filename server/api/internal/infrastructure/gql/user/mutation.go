package user

import (
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gql/gqlmodel"
)

type updateMeMutation struct {
	UpdateMe struct {
		Me gqlmodel.Me `graphql:"me"`
	} `graphql:"updateMe(input: $input)"`
}

type signupOIDCMutation struct {
	SignupOIDC struct {
		User gqlmodel.User `graphql:"user"`
	} `graphql:"signupOIDC(input: $input)"`
}
