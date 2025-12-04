package interactor

import (
	"context"

	gqluser "github.com/reearth/reearth-accounts/server/pkg/gqlclient/user"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
)

type User struct {
	userRepo gqluser.UserRepo
}

func NewUser(userRepo gqluser.UserRepo) interfaces.User {
	return &User{
		userRepo: userRepo,
	}
}

func (i *User) FindByIDs(ctx context.Context, ids accountsid.UserIDList) (accountsuser.List, error) {
	return i.userRepo.FindByIDs(ctx, ids)
}

func (i *User) UserByNameOrEmail(ctx context.Context, nameOrEmail string) (*accountsuser.User, error) {
	return i.userRepo.UserByNameOrEmail(ctx, nameOrEmail)
}

func (i *User) UpdateMe(ctx context.Context, p interfaces.UpdateMeParam) (*accountsuser.User, error) {
	attrs := accountsuser.UpdateAttrs{
		Name:                 p.Name,
		Email:                p.Email,
		Lang:                 p.Lang,
		Password:             p.Password,
		PasswordConfirmation: p.PasswordConfirmation,
	}
	return i.userRepo.UpdateMe(ctx, attrs)
}

func (i *User) Signup(ctx context.Context, p interfaces.SignupParam) (*accountsuser.User, error) {
	attrs := accountsuser.SignupAttrs{
		ID:          p.UserID,
		WorkspaceID: p.WorkspaceID,
		Name:        p.Name,
		Email:       p.Email,
		Password:    p.Password,
		Secret:      p.Secret,
		Lang:        p.Lang,
		Theme:       p.Theme,
		MockAuth:    p.MockAuth,
	}
	return i.userRepo.Signup(ctx, attrs)
}

func (i *User) SignupOIDC(ctx context.Context, p interfaces.SignupOIDCParam) (*accountsuser.User, error) {
	attrs := accountsuser.SignupOIDCAttrs{
		UserID:      p.UserID,
		Name:        p.Name,
		Email:       p.Email,
		Sub:         p.Sub,
		Lang:        p.Lang,
		WorkspaceID: p.WorkspaceID,
		Secret:      p.Secret,
	}
	return i.userRepo.SignupOIDC(ctx, attrs)
}

func (i *User) RemoveMyAuth(ctx context.Context, authProvider string) (*accountsuser.User, error) {
	return i.userRepo.RemoveMyAuth(ctx, authProvider)
}

func (i *User) DeleteMe(ctx context.Context, uid accountsid.UserID) error {
	return i.userRepo.DeleteMe(ctx, uid)
}

func (i *User) CreateVerification(ctx context.Context, email string) error {
	return i.userRepo.CreateVerification(ctx, email)
}

func (i *User) VerifyUser(ctx context.Context, code string) (*accountsuser.User, error) {
	return i.userRepo.VerifyUser(ctx, code)
}

func (i *User) StartPasswordReset(ctx context.Context, email string) error {
	return i.userRepo.StartPasswordReset(ctx, email)
}

func (i *User) PasswordReset(ctx context.Context, password string, token string) error {
	return i.userRepo.PasswordReset(ctx, password, token)
}
