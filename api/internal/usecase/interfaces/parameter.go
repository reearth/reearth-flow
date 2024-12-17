package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
)

type DeclareParameterInput struct {
	Index     *int // Optional, will be set to last position if nil
	Name      string
	ProjectID id.ProjectID
	Required  bool
	Type      parameter.Type
	Value     interface{}
}

type UpdateParameterOrderInput struct {
	NewIndex  int
	ParamID   id.ParameterID
	ProjectID id.ProjectID
}

type UpdateParameterValueInput struct {
	ParamID id.ParameterID
	Value   interface{}
}

type Parameter interface {
	DeclareParameter(context.Context, DeclareParameterInput, *usecase.Operator) (*parameter.Parameter, error)
	Fetch(context.Context, id.ParameterIDList, *usecase.Operator) (*parameter.ParameterList, error)
	FetchByProject(context.Context, id.ProjectID, *usecase.Operator) (*parameter.ParameterList, error)
	RemoveParameter(context.Context, id.ParameterID, *usecase.Operator) (id.ParameterID, error)
	UpdateParameterOrder(context.Context, UpdateParameterOrderInput, *usecase.Operator) (*parameter.ParameterList, error)
	UpdateParameterValue(context.Context, UpdateParameterValueInput, *usecase.Operator) (*parameter.Parameter, error)
}
