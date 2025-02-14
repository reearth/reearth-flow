package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
)

type DeclareParameterParam struct {
	Index     *int // Optional, will be set to last position if nil
	Name      string
	ProjectID id.ProjectID
	Required  bool
	Type      parameter.Type
	Value     interface{}
}

type UpdateParameterOrderParam struct {
	NewIndex  int
	ParamID   id.ParameterID
	ProjectID id.ProjectID
}

type UpdateParameterValueParam struct {
	ParamID id.ParameterID
	Value   interface{}
}

type Parameter interface {
	DeclareParameter(context.Context, DeclareParameterParam) (*parameter.Parameter, error)
	Fetch(context.Context, id.ParameterIDList) (*parameter.ParameterList, error)
	FetchByProject(context.Context, id.ProjectID) (*parameter.ParameterList, error)
	RemoveParameter(context.Context, id.ParameterID) (id.ParameterID, error)
	UpdateParameterOrder(context.Context, UpdateParameterOrderParam) (*parameter.ParameterList, error)
	UpdateParameterValue(context.Context, UpdateParameterValueParam) (*parameter.Parameter, error)
}
