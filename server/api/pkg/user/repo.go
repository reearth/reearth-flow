//go:generate go run go.uber.org/mock/mockgen@latest -source=repo.go -destination=mockrepo/mockrepo.go -package=mockrepo -mock_names=Repo=MockUserRepo
package user

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
)

type Repo interface {
	FindMe(ctx context.Context) (*User, error)
	FindByIDs(ctx context.Context, ids id.UserIDList) (List, error)
	UserByNameOrEmail(ctx context.Context, nameOrEmail string) (*User, error)
	UpdateMe(ctx context.Context, attrs UpdateAttrs) (*User, error)
	Signup(ctx context.Context, attrs SignupAttrs) (*User, error)
	SignupOIDC(ctx context.Context, attrs SignupOIDCAttrs) (*User, error)
	RemoveMyAuth(ctx context.Context, authProvider string) (*User, error)
	DeleteMe(ctx context.Context, uid id.UserID) error
	CreateVerification(ctx context.Context, email string) error
	VerifyUser(ctx context.Context, code string) (*User, error)
	StartPasswordReset(ctx context.Context, email string) error
	PasswordReset(ctx context.Context, password string, token string) error
}
