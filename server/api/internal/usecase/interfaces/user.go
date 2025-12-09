package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/user"
	"github.com/reearth/reearth-flow/api/pkg/workspace"
	"golang.org/x/text/language"
)

type UpdateMeParam struct {
	Name                 *string
	Email                *string
	Lang                 *language.Tag
	Password             *string
	PasswordConfirmation *string
}

type SignupParam struct {
	Secret      *string
	Lang        *language.Tag
	Theme       *user.Theme
	UserID      *user.ID
	WorkspaceID *workspace.ID
	Email       string
	Name        string
	Password    string
	MockAuth    bool
}

type SignupOIDCParam struct {
	UserID      *id.UserID
	Name        *string
	Email       *string
	Sub         *string
	Lang        *language.Tag
	WorkspaceID *id.WorkspaceID
	Secret      *string
}

type User interface {
	FindByIDs(context.Context, id.UserIDList) (user.List, error)
	UserByNameOrEmail(context.Context, string) (*user.User, error)
	UpdateMe(context.Context, UpdateMeParam) (*user.User, error)
	Signup(context.Context, SignupParam) (*user.User, error)
	SignupOIDC(context.Context, SignupOIDCParam) (*user.User, error)
	RemoveMyAuth(context.Context, string) (*user.User, error)
	DeleteMe(context.Context, id.UserID) error
	CreateVerification(context.Context, string) error
	VerifyUser(context.Context, string) (*user.User, error)
	StartPasswordReset(context.Context, string) error
	PasswordReset(context.Context, string, string) error
}
