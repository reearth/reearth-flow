package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
)

type Parameter interface {
	FindByID(context.Context, id.ParameterID) (*parameter.Parameter, error)
	FindByIDs(context.Context, id.ParameterIDList) (*parameter.ParameterList, error)
	FindByProject(context.Context, id.ProjectID) (*parameter.ParameterList, error)
	Remove(context.Context, id.ParameterID) error
	RemoveAll(context.Context, id.ParameterIDList) error
	RemoveAllByProject(context.Context, id.ProjectID) error
	Save(context.Context, *parameter.Parameter) error
}
