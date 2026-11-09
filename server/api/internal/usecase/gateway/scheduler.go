package gateway

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"google.golang.org/protobuf/types/known/timestamppb"
)

type Scheduler interface {
	CreateScheduledJob(ctx context.Context, t *trigger.Trigger) error
	UpdateScheduledJob(ctx context.Context, t *trigger.Trigger) error
	DeleteScheduledJob(ctx context.Context, triggerID id.TriggerID) error
	// PauseScheduledJob stops a trigger's job firing until it is resumed.
	// CreateScheduledJob and UpdateScheduledJob already pause the job of a
	// disabled trigger and resume an enabled one's.
	PauseScheduledJob(ctx context.Context, triggerID id.TriggerID) error
	GetScheduledJob(ctx context.Context, triggerID id.TriggerID) (*ScheduledJobInfo, error)
	// ListScheduledJobs returns every trigger job in the scheduler's location.
	ListScheduledJobs(ctx context.Context) ([]ScheduledTriggerJob, error)
	Close() error
}

// ScheduledTriggerJob is the scheduler job of a time-driven trigger.
type ScheduledTriggerJob struct {
	TriggerID id.TriggerID
	// Authenticated is whether the job sends the OIDC token /execute-scheduled
	// requires. A job created before that requirement does not.
	Authenticated bool
}

type ScheduledJobInfo struct {
	LastAttempt  *timestamppb.Timestamp
	NextSchedule *timestamppb.Timestamp
	Name         string
	Schedule     string
	State        ScheduledJobState
}

type ScheduledJobState string

const (
	ScheduledJobStateEnabled  ScheduledJobState = "ENABLED"
	ScheduledJobStatePaused   ScheduledJobState = "PAUSED"
	ScheduledJobStateDisabled ScheduledJobState = "DISABLED"
	ScheduledJobStateUnknown  ScheduledJobState = "UNKNOWN"
)
