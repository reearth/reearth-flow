package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
)

func (r *mutationResolver) CreateTrigger(ctx context.Context, input gqlmodel.CreateTriggerInput) (*gqlmodel.Trigger, error) {
	wsid, err := gqlmodel.ToID[accountdomain.Workspace](input.WorkspaceID)
	if err != nil {
		return nil, err
	}

	did, err := gqlmodel.ToID[id.Deployment](input.DeploymentID)
	if err != nil {
		return nil, err
	}

	var param interfaces.CreateTriggerParam
	param.WorkspaceID = wsid
	param.DeploymentID = did

	param.Description = input.Description

	if input.TimeDriverInput != nil {
		param.EventSource = "TIME_DRIVEN"
		param.TimeInterval = gqlmodel.FromTimeInterval(input.TimeDriverInput.Interval)
	} else if input.APIDriverInput != nil {
		param.EventSource = "API_DRIVEN"
		param.AuthToken = input.APIDriverInput.Token
	}

	res, err := usecases(ctx).Trigger.Create(ctx, param)
	if err != nil {
		return nil, err
	}

	return gqlmodel.ToTrigger(res), nil
}

func (r *mutationResolver) UpdateTrigger(ctx context.Context, input gqlmodel.UpdateTriggerInput) (*gqlmodel.Trigger, error) {
	tid, err := gqlmodel.ToID[id.Trigger](input.TriggerID)
	if err != nil {
		return nil, err
	}

	param := interfaces.UpdateTriggerParam{
		ID:          tid,
		Description: input.Description,
	}

	if input.DeploymentID != nil {
		did, err := gqlmodel.ToID[id.Deployment](*input.DeploymentID)
		if err != nil {
			return nil, err
		}
		param.DeploymentID = &did
	}

	if input.TimeDriverInput != nil {
		param.EventSource = "TIME_DRIVEN"
		param.TimeInterval = gqlmodel.FromTimeInterval(input.TimeDriverInput.Interval)
	} else if input.APIDriverInput != nil {
		param.EventSource = "API_DRIVEN"
		param.AuthToken = input.APIDriverInput.Token
	}

	res, err := usecases(ctx).Trigger.Update(ctx, param)
	if err != nil {
		return nil, err
	}

	return gqlmodel.ToTrigger(res), nil
}

func (r *mutationResolver) DeleteTrigger(ctx context.Context, triggerId gqlmodel.ID) (bool, error) {
	tid, err := gqlmodel.ToID[id.Trigger](triggerId)
	if err != nil {
		return false, err
	}

	err = usecases(ctx).Trigger.Delete(ctx, tid)
	if err != nil {
		return false, err
	}

	return true, nil
}
