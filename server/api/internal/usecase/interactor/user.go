package interactor

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/user"
)

type User struct {
	userRepo user.Repo
}

func NewUser(userRepo user.Repo) interfaces.User {
	return &User{
		userRepo: userRepo,
	}
}

func (i *User) FindByIDs(ctx context.Context, ids id.UserIDList) (user.List, error) {
	return i.userRepo.FindByIDs(ctx, ids)
}

func (i *User) UserByNameOrEmail(ctx context.Context, nameOrEmail string) (*user.User, error) {
	return i.userRepo.UserByNameOrEmail(ctx, nameOrEmail)
}

func (i *User) UpdateMe(ctx context.Context, p interfaces.UpdateMeParam) (*user.User, error) {
	attrs := user.UpdateAttrs{
		Name:                 p.Name,
		Email:                p.Email,
		Lang:                 p.Lang,
		Password:             p.Password,
		PasswordConfirmation: p.PasswordConfirmation,
	}
	return i.userRepo.UpdateMe(ctx, attrs)
}
