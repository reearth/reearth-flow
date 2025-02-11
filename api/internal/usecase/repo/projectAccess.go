package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/projectAccess"
)

type ProjectAccess interface {
	FindByProjectID(context.Context, id.ProjectID) (*projectAccess.ProjectAccess, error)
	FindByToken(context.Context, string) (*projectAccess.ProjectAccess, error)
	Save(context.Context, *projectAccess.ProjectAccess) error
}
