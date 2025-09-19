package user

import (
	"github.com/hasura/go-graphql-client"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/gql/gqlmodel"
)

type updateMeMutation struct {
	UpdateMe struct {
		Me gqlmodel.Me `graphql:"me"`
	} `graphql:"updateMe(input: $input)"`
}

type signupMutation struct {
	Signup struct {
		User gqlmodel.User `graphql:"user"`
	} `graphql:"signup(input: $input)"`
}

type signupOIDCMutation struct {
	SignupOIDC struct {
		User gqlmodel.User `graphql:"user"`
	} `graphql:"signupOIDC(input: $input)"`
}

type removeMyAuthMutation struct {
	RemoveMyAuth struct {
		Me gqlmodel.Me `graphql:"me"`
	} `graphql:"removeMyAuth(input: $input)"`
}

type deleteMeMutation struct {
	DeleteMe struct {
		ID graphql.ID `graphql:"userId"`
	} `graphql:"deleteMe(input: $input)"`
}

type createVerificationMutation struct {
	CreateVerification graphql.Boolean `graphql:"createVerification(input: $input)"`
}

type verifyUserMutation struct {
	VerifyUser struct {
		User gqlmodel.User `graphql:"user"`
	} `graphql:"verifyUser(input: $input)"`
}

type startPasswordResetMutation struct {
	StartPasswordReset graphql.Boolean `graphql:"startPasswordReset(input: $input)"`
}

type passwordResetMutation struct {
	PasswordReset graphql.Boolean `graphql:"passwordReset(input: $input)"`
}
