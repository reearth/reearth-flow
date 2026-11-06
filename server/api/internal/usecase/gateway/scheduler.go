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
	Close() error
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
