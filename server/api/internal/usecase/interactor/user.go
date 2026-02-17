package interactor

import (
	"context"

	gqluser "github.com/reearth/reearth-accounts/server/pkg/gqlclient/user"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/samber/lo"
)

type User struct {
	userRepo gqluser.Repo
}

func NewUser(userRepo gqluser.Repo) interfaces.User {
	return &User{
		userRepo: userRepo,
	}
}

func (i *User) FindByIDs(ctx context.Context, ids accountsid.UserIDList) (accountsuser.List, error) {
	return i.userRepo.FindByIDs(ctx, ids.Strings())
}

func (i *User) UserByNameOrEmail(ctx context.Context, nameOrEmail string) (*accountsuser.User, error) {
	return i.userRepo.FindByNameOrEmail(ctx, nameOrEmail)
}

func (i *User) UpdateMe(ctx context.Context, p interfaces.UpdateMeParam) (*accountsuser.User, error) {
	var langPtr *string
	if p.Lang != nil {
		l := p.Lang.String()
		langPtr = &l
	}

	input := gqluser.UpdateMeInput{
		Name:                 p.Name,
		Email:                p.Email,
		Lang:                 langPtr,
		Password:             p.Password,
		PasswordConfirmation: p.PasswordConfirmation,
	}

	return i.userRepo.UpdateMe(ctx, input)
}

func (i *User) Signup(ctx context.Context, p interfaces.SignupParam) (*accountsuser.User, error) {
	userID := ""
	if p.UserID != nil {
		userID = p.UserID.String()
	}

	workspaceID := ""
	if p.WorkspaceID != nil {
		workspaceID = p.WorkspaceID.String()
	}

	if userID == "" {
		return i.userRepo.SignupNoID(
			ctx,
			p.Name,
			p.Email,
			p.Password,
			lo.FromPtr(p.Secret),
			p.MockAuth,
		)
	}

	return i.userRepo.Signup(
		ctx,
		userID,
		p.Name,
		p.Email,
		p.Password,
		lo.FromPtr(p.Secret),
		workspaceID,
		p.MockAuth,
	)
}

func (i *User) SignupOIDC(ctx context.Context, p interfaces.SignupOIDCParam) (*accountsuser.User, error) {
	return i.userRepo.SignupOIDC(
		ctx,
		lo.FromPtr(p.Name),
		lo.FromPtr(p.Email),
		lo.FromPtr(p.Sub),
		lo.FromPtr(p.Secret),
	)
}

func (i *User) RemoveMyAuth(ctx context.Context, authProvider string) (*accountsuser.User, error) {
	return i.userRepo.RemoveMyAuth(ctx, authProvider)
}

func (i *User) DeleteMe(ctx context.Context, uid accountsid.UserID) error {
	return i.userRepo.DeleteMe(ctx, uid.String())
}

func (i *User) CreateVerification(ctx context.Context, email string) error {
	_, err := i.userRepo.CreateVerification(ctx, email)
	return err
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
