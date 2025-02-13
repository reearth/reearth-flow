package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
)

type ProjectAccess interface {
	Fetch(context.Context, string) (*project.Project, error)
	Share(context.Context, id.ProjectID) (string, error)
	Unshare(context.Context, id.ProjectID) error
}
