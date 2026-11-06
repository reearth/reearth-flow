package interfaces

import (
	"context"
	"errors"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearth-flow/api/pkg/variable"
)

type CreateTriggerParam struct {
	Description  string
	EventSource  trigger.EventSourceType
	TimeInterval trigger.TimeInterval
	Variables    []variable.Variable
	WorkspaceID  accountsid.WorkspaceID
	DeploymentID id.DeploymentID
	Enabled      bool
}

type ExecuteAPITriggerParam struct {
	NotificationURL     *string
	Variables           map[string]interface{}
	AuthenticationToken string
	TriggerID           id.TriggerID
}

type ExecuteTimeDrivenTriggerParam struct {
	TriggerID id.TriggerID
}

type UpdateTriggerParam struct {
	DeploymentID *id.DeploymentID
	Description  *string
	Enabled      *bool
	EventSource  trigger.EventSourceType
	TimeInterval trigger.TimeInterval
	Variables    []variable.Variable
	ID           id.TriggerID
}

var (
	ErrTriggerNotFound     error = errors.New("trigger not found")
	ErrInvalidEventSource  error = errors.New("invalid event source type")
	ErrInvalidTimeInterval error = errors.New("invalid time interval")
	ErrInvalidTriggerInput error = errors.New("only a time-driven trigger has a time interval")
	// ErrDeploymentNotInWorkspace refuses a trigger that points at a deployment
	// in another workspace: its runs would be checked and billed against one
	// workspace while the job lands in the other.
	ErrDeploymentNotInWorkspace error = errors.New("deployment does not belong to the trigger's workspace")

	ErrHourlyScheduleNotAllowed error = errors.New("this workspace's plan does not allow hourly schedules")

	ErrTriggerNotAPIDriven error = errors.New("only an API-driven trigger has an auth token")

	ErrTriggerDisabled      error = errors.New("trigger is disabled")
	ErrTriggerNotTimeDriven error = errors.New("trigger is not time-driven")
)

// ErrAPITriggerDailyLimit refuses an API trigger call on a free workspace that
// has used its calls for the day.
var ErrAPITriggerDailyLimit error = errors.New("this workspace's plan does not allow more API trigger calls today")

// APITriggerLimitError is ErrAPITriggerDailyLimit with the time the count
// resets, so the webhook can tell the caller when to retry.
type APITriggerLimitError struct {
	ResetAt time.Time
}

func (e *APITriggerLimitError) Error() string { return ErrAPITriggerDailyLimit.Error() }

func (e *APITriggerLimitError) Unwrap() error { return ErrAPITriggerDailyLimit }

type Trigger interface {
	ExecuteAPITrigger(context.Context, ExecuteAPITriggerParam) (*job.Job, error)
	ExecuteTimeDrivenTrigger(context.Context, ExecuteTimeDrivenTriggerParam) (*job.Job, error)
	Fetch(context.Context, []id.TriggerID) ([]*trigger.Trigger, error)
	FindByID(context.Context, id.TriggerID) (*trigger.Trigger, error)
	FindByWorkspace(context.Context, accountsid.WorkspaceID, *PaginationParam, *string) ([]*trigger.Trigger, *PageBasedInfo, error)
	// Create, Update and RegenerateAuthToken return authToken only when they
	// issued the trigger a new token. It is never returned again.
	Create(context.Context, CreateTriggerParam) (_ *trigger.Trigger, authToken string, _ error)
	Update(context.Context, UpdateTriggerParam) (_ *trigger.Trigger, authToken string, _ error)
	RegenerateAuthToken(context.Context, id.TriggerID) (_ *trigger.Trigger, authToken string, _ error)
	Delete(context.Context, id.TriggerID) error
}
