package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

func (r *mutationResolver) DeclareParameter(ctx context.Context, projectID gqlmodel.ID, input gqlmodel.DeclareParameterInput) (*gqlmodel.Parameter, error) {
	pid, err := gqlmodel.ToID[id.Project](projectID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Parameter.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		Index:        input.Index,
		Name:         input.Name,
		ProjectID:    pid,
		Required:     input.Required,
		Public:       input.Public,
		Type:         gqlmodel.FromParameterType(input.Type),
		DefaultValue: input.DefaultValue,
	})
	if err != nil {
		return nil, err
	}

	return gqlmodel.ToParameter(res), nil
}

func (r *mutationResolver) UpdateParameter(ctx context.Context, paramID gqlmodel.ID, input gqlmodel.UpdateParameterInput) (*gqlmodel.Parameter, error) {
	pid, err := gqlmodel.ToID[id.Parameter](paramID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Parameter.UpdateParameter(ctx, interfaces.UpdateParameterParam{
		ParamID:       pid,
		DefaultValue:  &input.DefaultValue,
		PublicValue:   input.Public,
		RequiredValue: input.Required,
		NameValue:     input.Name,
		TypeValue:     gqlmodel.FromParameterType(input.Type),
	})
	if err != nil {
		return nil, err
	}

	return gqlmodel.ToParameter(res), nil
}

func (r *mutationResolver) UpdateParameterOrder(ctx context.Context, projectID gqlmodel.ID, input gqlmodel.UpdateParameterOrderInput) ([]*gqlmodel.Parameter, error) {
	pid, err := gqlmodel.ToID[id.Project](projectID)
	if err != nil {
		return nil, err
	}

	paramID, err := gqlmodel.ToID[id.Parameter](input.ParamID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).Parameter.UpdateParameterOrder(ctx, interfaces.UpdateParameterOrderParam{
		NewIndex:  input.NewIndex,
		ParamID:   paramID,
		ProjectID: pid,
	})
	if err != nil {
		return nil, err
	}

	return gqlmodel.ToParameters(res), nil
}

func (r *mutationResolver) RemoveParameter(ctx context.Context, input gqlmodel.RemoveParameterInput) (bool, error) {
	pid, err := gqlmodel.ToID[id.Parameter](input.ParamID)
	if err != nil {
		return false, err
	}

	_, err = usecases(ctx).Parameter.RemoveParameter(ctx, pid)
	if err != nil {
		return false, err
	}

	return true, nil
}
