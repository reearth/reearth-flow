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
	"github.com/reearth/reearthx/rerror"
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
	// ScheduledAt is the scheduled time the scheduler calls for, the same on
	// every retry of one run. With it, the trigger starts at most one job for
	// the time; without it, every call starts one.
	ScheduledAt *time.Time
	TriggerID   id.TriggerID
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
	ErrInvalidTriggerToken error = errors.New("invalid auth token")
	ErrInvalidEventSource  error = errors.New("invalid event source type")
	ErrInvalidTimeInterval error = errors.New("invalid time interval")
	ErrInvalidTriggerInput error = errors.New("only a time-driven trigger has a time interval")
	// ErrDeploymentNotInWorkspace refuses a trigger that points at a deployment
	// in another workspace: its runs would be checked and billed against one
	// workspace while the job lands in the other.
	ErrDeploymentNotInWorkspace error = errors.New("deployment does not belong to the trigger's workspace")

	ErrHourlyScheduleNotAllowed error = errors.New("this workspace's plan does not allow hourly schedules")

	// ErrEnabledTriggerLimit refuses enabling a trigger on a workspace that
	// already has as many triggers enabled as its plan allows.
	ErrEnabledTriggerLimit error = errors.New("this workspace already has as many triggers enabled as its plan allows")

	// ErrTriggerDisabled refuses running a trigger that is disabled.
	ErrTriggerDisabled error = errors.New("trigger is disabled")

	ErrTriggerNotAPIDriven error = errors.New("only an API-driven trigger has an auth token")

	// ErrTriggerScheduleNotUpdated is an edit of a time-driven trigger that
	// was saved, but whose Cloud Scheduler job could not be made to match it.
	// Saving the trigger again sends its whole job, so it can be retried.
	ErrTriggerScheduleNotUpdated error = errors.New("the trigger was saved, but its schedule could not be updated; save it again to retry")

	ErrTriggerNotTimeDriven error = errors.New("trigger is not time-driven")

	// ErrScheduledRunInProgress refuses a second job for a scheduled time a
	// time-driven trigger already has one for, which has not been submitted
	// yet. The call can be retried once that job is submitted or fails.
	ErrScheduledRunInProgress error = errors.New("a job for this scheduled run is already being started")
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

// ScheduledRunRefusal names why err refused a scheduled run, and reports
// whether a retry from the scheduler can succeed. Only errors known to be
// permanent are not retried, so an error added later is retried until it is
// listed here. A plan limit that resets with a period, such as a number of
// runs per day, belongs here: retries end long before it resets. A limit on
// runs at the same time does not, as a run ends.
func ScheduledRunRefusal(err error) (trigger.RefusalCode, bool) {
	switch {
	case errors.Is(err, ErrTriggerNotFound), errors.Is(err, rerror.ErrNotFound):
		return trigger.RefusalNotFound, false
	case errors.Is(err, ErrTriggerDisabled):
		return trigger.RefusalTriggerDisabled, false
	case errors.Is(err, ErrTriggerNotTimeDriven):
		return trigger.RefusalTriggerNotTimeDriven, false
	case errors.Is(err, ErrRequiredVariableMissing),
		errors.Is(err, ErrInvalidVariableValue),
		errors.Is(err, ErrUnknownVariable):
		return trigger.RefusalInvalidVariables, false
	case errors.Is(err, ErrOperationDenied):
		return trigger.RefusalNotAllowed, false
	case errors.Is(err, ErrTimeDrivenTriggerRunsPerDayLimit):
		return trigger.RefusalTimeDrivenTriggerRunsDailyLimit, false
	}
	return trigger.RefusalServerError, true
}
