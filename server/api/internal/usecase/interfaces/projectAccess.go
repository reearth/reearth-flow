package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/project"
)

type ProjectAccess interface {
	Fetch(context.Context, string) (*project.Project, error)
	FetchParameters(context.Context, string) (*parameter.ParameterList, error)
	Share(context.Context, id.ProjectID) (string, error)
	Unshare(context.Context, id.ProjectID) error
}
