package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workspace"
)

type Workspace interface {
	FindByIDs(context.Context, id.WorkspaceIDList) (workspace.List, error)
	FindByUser(context.Context, id.UserID) (workspace.List, error)
}
