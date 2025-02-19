package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
)

type ProjectAccess interface {
	Fetch(context.Context, string) (*project.Project, error)
	Share(context.Context, id.ProjectID, *usecase.Operator) (string, error)
	Unshare(context.Context, id.ProjectID, *usecase.Operator) error
}
