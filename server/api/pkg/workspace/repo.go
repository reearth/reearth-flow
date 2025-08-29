//go:generate go run go.uber.org/mock/mockgen@latest -source=repo.go -destination=mockrepo/mockrepo.go -package=mockrepo -mock_names=Repo=MockWorkspaceRepo
package workspace

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
)

type Repo interface {
	FindByIDs(ctx context.Context, ids id.WorkspaceIDList) (List, error)
	FindByUser(ctx context.Context, uid id.UserID) (List, error)
	Create(ctx context.Context, name string) (*Workspace, error)
	Update(ctx context.Context, wid id.WorkspaceID, name string) (*Workspace, error)
}
