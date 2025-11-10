package interfaces

import (
	"context"
	"errors"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
)

type CreateTriggerParam struct {
	WorkspaceID  id.WorkspaceID
	DeploymentID id.DeploymentID
	Description  string
	EventSource  trigger.EventSourceType
	TimeInterval trigger.TimeInterval
	AuthToken    string
	Variables    map[string]string
}

type ExecuteAPITriggerParam struct {
	AuthenticationToken string
	TriggerID           id.TriggerID
	NotificationURL     *string
	Variables           map[string]string
}

type ExecuteTimeDrivenTriggerParam struct {
	TriggerID id.TriggerID
	Variables map[string]string
}

type UpdateTriggerParam struct {
	ID           id.TriggerID
	DeploymentID *id.DeploymentID
	Description  *string
	EventSource  trigger.EventSourceType
	TimeInterval trigger.TimeInterval
	AuthToken    string
	Variables    map[string]string
}

var (
	ErrTriggerNotFound     error = errors.New("trigger not found")
	ErrInvalidEventSource  error = errors.New("invalid event source type")
	ErrInvalidTimeInterval error = errors.New("invalid time interval")
	ErrInvalidTriggerInput error = errors.New("either time interval or auth token must be provided")
)

type Trigger interface {
	ExecuteAPITrigger(context.Context, ExecuteAPITriggerParam) (*job.Job, error)
	ExecuteTimeDrivenTrigger(context.Context, ExecuteTimeDrivenTriggerParam) (*job.Job, error)
	Fetch(context.Context, []id.TriggerID) ([]*trigger.Trigger, error)
	FindByID(context.Context, id.TriggerID) (*trigger.Trigger, error)
	FindByWorkspace(context.Context, id.WorkspaceID, *PaginationParam, *string) ([]*trigger.Trigger, *PageBasedInfo, error)
	Create(context.Context, CreateTriggerParam) (*trigger.Trigger, error)
	Update(context.Context, UpdateTriggerParam) (*trigger.Trigger, error)
	Delete(context.Context, id.TriggerID) error
}
