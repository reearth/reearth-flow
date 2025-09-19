package user

import (
	"github.com/hasura/go-graphql-client"
)

type UpdateMeInput struct {
	Name                 *graphql.String `json:"name,omitempty"`
	Email                *graphql.String `json:"email,omitempty"`
	Lang                 *graphql.String `json:"lang,omitempty"`
	Password             *graphql.String `json:"password,omitempty"`
	PasswordConfirmation *graphql.String `json:"passwordConfirmation,omitempty"`
}

type SignupOIDCInput struct {
	ID          *graphql.ID     `json:"id,omitempty"`
	Lang        *graphql.String `json:"lang,omitempty"`
	WorkspaceID *graphql.ID     `json:"workspaceId,omitempty"`
	Secret      *graphql.String `json:"secret,omitempty"`
}

type RemoveMyAuthInput struct {
	Auth graphql.String `json:"auth"`
}

type DeleteMeInput struct {
	ID graphql.ID `json:"userId"`
}
