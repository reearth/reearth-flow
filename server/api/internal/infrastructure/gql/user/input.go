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

type SignupInput struct {
	ID          *graphql.ID      `json:"id,omitempty"`
	WorkspaceID *graphql.ID      `json:"workspaceID,omitempty"`
	Secret      *graphql.String  `json:"secret,omitempty"`
	Lang        *graphql.String  `json:"lang,omitempty"`
	Theme       *graphql.String  `json:"theme,omitempty"`
	MockAuth    *graphql.Boolean `json:"mockAuth,omitempty"`
	Name        graphql.String   `json:"name"`
	Email       graphql.String   `json:"email"`
	Password    graphql.String   `json:"password"`
}

type SignupOIDCInput struct {
	ID          *graphql.ID     `json:"id,omitempty"`
	Name        *graphql.String `json:"name"`
	Email       *graphql.String `json:"email"`
	Sub         *graphql.String `json:"sub"`
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

type CreateVerificationInput struct {
	Email graphql.String `json:"email"`
}

type VerifyUserInput struct {
	Code graphql.String `json:"code"`
}

type StartPasswordResetInput struct {
	Email graphql.String `json:"email"`
}

type PasswordResetInput struct {
	Password graphql.String `json:"password"`
	Token    graphql.String `json:"token"`
}
