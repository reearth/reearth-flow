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
