package gql

import (
	"context"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
)

func (r *mutationResolver) CreateTrigger(ctx context.Context, input gqlmodel.CreateTriggerInput) (*gqlmodel.TriggerPayload, error) {
	wsid, err := gqlmodel.ToID[accountsid.Workspace](input.WorkspaceID)
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
	param.Enabled = input.Enabled

	if input.Variables != nil {
		param.Variables, err = gqlmodel.FromVariables(input.Variables)
		if err != nil {
			return nil, err
		}
	}

	param.EventSource, param.TimeInterval, err = triggerEventSource(&input.EventSource, input.TimeDriverInput)
	if err != nil {
		return nil, err
	}

	res, authToken, err := usecases(ctx).Trigger.Create(ctx, param)
	if err != nil {
		return nil, err
	}

	return gqlmodel.ToTriggerPayload(res, authToken), nil
}

func (r *mutationResolver) UpdateTrigger(ctx context.Context, input gqlmodel.UpdateTriggerInput) (*gqlmodel.TriggerPayload, error) {
	tid, err := gqlmodel.ToID[id.Trigger](input.TriggerID)
	if err != nil {
		return nil, err
	}

	param := interfaces.UpdateTriggerParam{
		ID:          tid,
		Description: input.Description,
		Enabled:     input.Enabled,
	}

	if input.DeploymentID != nil {
		did, err := gqlmodel.ToID[id.Deployment](*input.DeploymentID)
		if err != nil {
			return nil, err
		}
		param.DeploymentID = &did
	}

	param.EventSource, param.TimeInterval, err = triggerEventSource(input.EventSource, input.TimeDriverInput)
	if err != nil {
		return nil, err
	}

	if input.Variables != nil {
		param.Variables, err = gqlmodel.FromVariables(input.Variables)
		if err != nil {
			return nil, err
		}
	}

	res, authToken, err := usecases(ctx).Trigger.Update(ctx, param)
	if err != nil {
		return nil, err
	}

	return gqlmodel.ToTriggerPayload(res, authToken), nil
}

func (r *mutationResolver) RegenerateTriggerToken(ctx context.Context, triggerID gqlmodel.ID) (*gqlmodel.TriggerPayload, error) {
	tid, err := gqlmodel.ToID[id.Trigger](triggerID)
	if err != nil {
		return nil, err
	}

	res, authToken, err := usecases(ctx).Trigger.RegenerateAuthToken(ctx, tid)
	if err != nil {
		return nil, err
	}

	return gqlmodel.ToTriggerPayload(res, authToken), nil
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

// triggerEventSource is the event source and interval an input asks for. A
// time interval makes a trigger time-driven, so an update can change the
// interval on its own, and a time-driven trigger needs one.
func triggerEventSource(eventSource *gqlmodel.EventSourceType, timeDriver *gqlmodel.TimeDriverInput) (trigger.EventSourceType, trigger.TimeInterval, error) {
	if timeDriver != nil {
		if eventSource != nil && *eventSource != gqlmodel.EventSourceTypeTimeDriven {
			return "", "", interfaces.ErrInvalidTriggerInput
		}
		return trigger.EventSourceTypeTimeDriven, gqlmodel.FromTimeInterval(timeDriver.Interval), nil
	}
	if eventSource == nil {
		return "", "", nil
	}
	switch *eventSource {
	case gqlmodel.EventSourceTypeAPIDriven:
		return trigger.EventSourceTypeAPIDriven, "", nil
	case gqlmodel.EventSourceTypeTimeDriven:
		return "", "", interfaces.ErrInvalidTimeInterval
	}
	return "", "", interfaces.ErrInvalidEventSource
}
