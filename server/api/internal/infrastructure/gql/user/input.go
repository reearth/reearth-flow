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
