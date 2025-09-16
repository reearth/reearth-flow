package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/user"
	"golang.org/x/text/language"
)

type UpdateMeParam struct {
	Name                 *string
	Email                *string
	Lang                 *language.Tag
	Password             *string
	PasswordConfirmation *string
}

type SignupOIDCParam struct {
	UserID      *id.UserID
	Lang        *language.Tag
	WorkspaceID *id.WorkspaceID
	Secret      *string
}

type User interface {
	FindByIDs(context.Context, id.UserIDList) (user.List, error)
	UserByNameOrEmail(context.Context, string) (*user.User, error)
	UpdateMe(context.Context, UpdateMeParam) (*user.User, error)
	SignupOIDC(context.Context, SignupOIDCParam) (*user.User, error)
	RemoveMyAuth(context.Context, string) (*user.User, error)
}
