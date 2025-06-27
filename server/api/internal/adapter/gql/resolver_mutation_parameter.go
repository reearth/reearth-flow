package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

func convertJSONToInterface(val gqlmodel.JSON) interface{} {
	if val == nil {
		return nil
	}
	return map[string]interface{}(val)
}

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
		Config:       convertJSONToInterface(input.Config),
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
		DefaultValue:  input.DefaultValue,
		PublicValue:   input.Public,
		RequiredValue: input.Required,
		NameValue:     input.Name,
		TypeValue:     gqlmodel.FromParameterType(input.Type),
		Config:        convertJSONToInterface(input.Config),
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

func (r *mutationResolver) UpdateParameters(ctx context.Context, input gqlmodel.ParameterBatchInput) ([]*gqlmodel.Parameter, error) {
	pid, err := gqlmodel.ToID[id.Project](input.ProjectID)
	if err != nil {
		return nil, err
	}

	creates := make([]interfaces.DeclareParameterParam, len(input.Creates))
	for i, create := range input.Creates {
		creates[i] = interfaces.DeclareParameterParam{
			Index:        create.Index,
			Name:         create.Name,
			ProjectID:    pid,
			Required:     create.Required,
			Public:       create.Public,
			Type:         gqlmodel.FromParameterType(create.Type),
			DefaultValue: create.DefaultValue,
			Config:       convertJSONToInterface(create.Config),
		}
	}

	updates := make([]interfaces.UpdateParameterBatchItemParam, len(input.Updates))
	for i, update := range input.Updates {
		paramID, err := gqlmodel.ToID[id.Parameter](update.ParamID)
		if err != nil {
			return nil, err
		}

		updateParam := interfaces.UpdateParameterBatchItemParam{
			ParamID:      paramID,
			DefaultValue: update.DefaultValue,
			Config:       convertJSONToInterface(update.Config),
		}

		if update.Name != nil {
			updateParam.NameValue = update.Name
		}
		if update.Type != nil {
			paramType := gqlmodel.FromParameterType(*update.Type)
			updateParam.TypeValue = &paramType
		}
		if update.Required != nil {
			updateParam.RequiredValue = update.Required
		}
		if update.Public != nil {
			updateParam.PublicValue = update.Public
		}

		updates[i] = updateParam
	}

	deletes := make(id.ParameterIDList, len(input.Deletes))
	for i, deleteID := range input.Deletes {
		paramID, err := gqlmodel.ToID[id.Parameter](deleteID)
		if err != nil {
			return nil, err
		}
		deletes[i] = paramID
	}

	reorders := make([]interfaces.UpdateParameterOrderParam, len(input.Reorders))
	for i, reorder := range input.Reorders {
		paramID, err := gqlmodel.ToID[id.Parameter](reorder.ParamID)
		if err != nil {
			return nil, err
		}
		reorders[i] = interfaces.UpdateParameterOrderParam{
			NewIndex:  reorder.NewIndex,
			ParamID:   paramID,
			ProjectID: pid,
		}
	}

	res, err := usecases(ctx).Parameter.UpdateParameters(ctx, interfaces.UpdateParametersParam{
		ProjectID: pid,
		Creates:   creates,
		Updates:   updates,
		Deletes:   deletes,
		Reorders:  reorders,
	})
	if err != nil {
		return nil, err
	}

	return gqlmodel.ToParameters(res), nil
}

func (r *mutationResolver) RemoveParameters(ctx context.Context, input gqlmodel.RemoveParametersInput) (bool, error) {
	pids := make(id.ParameterIDList, len(input.ParamIds))
	for i, paramID := range input.ParamIds {
		pid, err := gqlmodel.ToID[id.Parameter](paramID)
		if err != nil {
			return false, err
		}
		pids[i] = pid
	}

	_, err := usecases(ctx).Parameter.RemoveParameters(ctx, pids)
	if err != nil {
		return false, err
	}

	return true, nil
}
