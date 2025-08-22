package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/user"
)

type User interface {
	FindByIDs(context.Context, id.UserIDList) (user.List, error)
	UserByNameOrEmail(context.Context, string) (*user.User, error)
}
