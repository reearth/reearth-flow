package interfaces

import (
	"context"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
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
	Email       string
	Name        string
	Password    string
	Secret      *string
	Lang        *language.Tag
	Theme       *accountsuser.Theme
	UserID      *accountsuser.ID
	WorkspaceID *accountsworkspace.ID
	MockAuth    bool
}

type SignupOIDCParam struct {
	UserID      *accountsid.UserID
	Name        *string
	Email       *string
	Sub         *string
	Lang        *language.Tag
	WorkspaceID *accountsid.WorkspaceID
	Secret      *string
}

type User interface {
	FindByIDs(context.Context, accountsid.UserIDList) (accountsuser.List, error)
	UserByNameOrEmail(context.Context, string) (*accountsuser.User, error)
	UpdateMe(context.Context, UpdateMeParam) (*accountsuser.User, error)
	Signup(context.Context, SignupParam) (*accountsuser.User, error)
	SignupOIDC(context.Context, SignupOIDCParam) (*accountsuser.User, error)
	RemoveMyAuth(context.Context, string) (*accountsuser.User, error)
	DeleteMe(context.Context, accountsid.UserID) error
	CreateVerification(context.Context, string) error
	VerifyUser(context.Context, string) (*accountsuser.User, error)
	StartPasswordReset(context.Context, string) error
	PasswordReset(context.Context, string, string) error
}
