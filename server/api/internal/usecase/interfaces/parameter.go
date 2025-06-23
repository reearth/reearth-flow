package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
)

type DeclareParameterParam struct {
	Index        *int // Optional, will be set to last position if nil
	Name         string
	ProjectID    id.ProjectID
	Required     bool
	Public       bool
	Type         parameter.Type
	DefaultValue any
}

type UpdateParameterOrderParam struct {
	NewIndex  int
	ParamID   id.ParameterID
	ProjectID id.ProjectID
}

type UpdateParameterParam struct {
	ParamID       id.ParameterID
	DefaultValue  any
	NameValue     string
	RequiredValue bool
	PublicValue   bool
	TypeValue     parameter.Type
}

type Parameter interface {
	DeclareParameter(context.Context, DeclareParameterParam) (*parameter.Parameter, error)
	Fetch(context.Context, id.ParameterIDList) (*parameter.ParameterList, error)
	FetchByProject(context.Context, id.ProjectID) (*parameter.ParameterList, error)
	RemoveParameter(context.Context, id.ParameterID) (id.ParameterID, error)
	RemoveParameters(context.Context, id.ParameterIDList) (id.ParameterIDList, error)
	UpdateParameterOrder(context.Context, UpdateParameterOrderParam) (*parameter.ParameterList, error)
	UpdateParameter(context.Context, UpdateParameterParam) (*parameter.Parameter, error)
}
