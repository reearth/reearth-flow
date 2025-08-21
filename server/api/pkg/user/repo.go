//go:generate go run go.uber.org/mock/mockgen@latest -source=repo.go -destination=mockrepo/mockrepo.go -package=mockrepo -mock_names=Repo=MockUserRepo
package user

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
)

type Repo interface {
	FindMe(ctx context.Context) (*User, error)
	FindByIDs(ctx context.Context, ids id.UserIDList) (List, error)
}
