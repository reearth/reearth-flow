package interfaces

import (
	"context"
	"errors"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearthx/account/accountdomain"
)

type CreateTriggerParam struct {
	WorkspaceID  accountdomain.WorkspaceID
	DeploymentID id.DeploymentID
	Description  string
	EventSource  trigger.EventSourceType
	TimeInterval trigger.TimeInterval
	AuthToken    string
}

type ExecuteAPITriggerParam struct {
	AuthenticationToken string
	TriggerID           id.TriggerID
	NotificationURL     *string
	Variables           map[string]interface{}
}

type UpdateTriggerParam struct {
	ID           id.TriggerID
	DeploymentID *id.DeploymentID
	Description  *string
	EventSource  trigger.EventSourceType
	TimeInterval trigger.TimeInterval
	AuthToken    string
}

var (
	ErrTriggerNotFound     error = errors.New("trigger not found")
	ErrInvalidEventSource  error = errors.New("invalid event source type")
	ErrInvalidTimeInterval error = errors.New("invalid time interval")
	ErrInvalidTriggerInput error = errors.New("either time interval or auth token must be provided")
)

type Trigger interface {
	ExecuteAPITrigger(context.Context, ExecuteAPITriggerParam, *usecase.Operator) (*job.Job, error)
	Fetch(context.Context, []id.TriggerID, *usecase.Operator) ([]*trigger.Trigger, error)
	FindByID(context.Context, id.TriggerID, *usecase.Operator) (*trigger.Trigger, error)
	FindByWorkspace(context.Context, accountdomain.WorkspaceID, *PaginationParam, *usecase.Operator) ([]*trigger.Trigger, *PageBasedInfo, error)
	Create(context.Context, CreateTriggerParam, *usecase.Operator) (*trigger.Trigger, error)
	Update(context.Context, UpdateTriggerParam, *usecase.Operator) (*trigger.Trigger, error)
	Delete(context.Context, id.TriggerID, *usecase.Operator) error
}
