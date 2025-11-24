package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
)

type DeclareParameterParam struct {
	DefaultValue any
	Config       any
	Index        *int
	Name         string
	Type         parameter.Type
	ProjectID    id.ProjectID
	Required     bool
	Public       bool
}

type UpdateParameterOrderParam struct {
	NewIndex  int
	ParamID   id.ParameterID
	ProjectID id.ProjectID
}

type UpdateParameterParam struct {
	DefaultValue  any
	Config        any
	NameValue     string
	TypeValue     parameter.Type
	ParamID       id.ParameterID
	RequiredValue bool
	PublicValue   bool
}

type UpdateParameterBatchItemParam struct {
	DefaultValue  any
	Config        any
	NameValue     *string
	RequiredValue *bool
	PublicValue   *bool
	TypeValue     *parameter.Type
	ParamID       id.ParameterID
}

type UpdateParametersParam struct {
	Creates   []DeclareParameterParam
	Updates   []UpdateParameterBatchItemParam
	Deletes   id.ParameterIDList
	Reorders  []UpdateParameterOrderParam
	ProjectID id.ProjectID
}

type Parameter interface {
	DeclareParameter(context.Context, DeclareParameterParam) (*parameter.Parameter, error)
	Fetch(context.Context, id.ParameterIDList) (*parameter.ParameterList, error)
	FetchByProject(context.Context, id.ProjectID) (*parameter.ParameterList, error)
	RemoveParameter(context.Context, id.ParameterID) (id.ParameterID, error)
	RemoveParameters(context.Context, id.ParameterIDList) (id.ParameterIDList, error)
	UpdateParameterOrder(context.Context, UpdateParameterOrderParam) (*parameter.ParameterList, error)
	UpdateParameter(context.Context, UpdateParameterParam) (*parameter.Parameter, error)
	UpdateParameters(context.Context, UpdateParametersParam) (*parameter.ParameterList, error)
}
