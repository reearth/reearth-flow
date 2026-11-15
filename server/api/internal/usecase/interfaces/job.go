package interfaces

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
)

// ErrJobNotCancellable refuses to cancel a job that has already finished.
var ErrJobNotCancellable error = errors.New("job cannot be cancelled")

// ErrJobAlreadyRunning refuses a run on a workspace that already has as many
// pending or running jobs as its plan allows.
var ErrJobAlreadyRunning error = errors.New("this workspace already has as many jobs running as its plan allows")

// JobLimitError is ErrJobAlreadyRunning with the jobs that hold the
// workspace's slots, so the user can tell which to wait for or cancel.
type JobLimitError struct {
	Running []id.JobID
}

func (e *JobLimitError) Error() string {
	if len(e.Running) == 0 {
		return ErrJobAlreadyRunning.Error()
	}
	ids := make([]string, len(e.Running))
	for i, jid := range e.Running {
		ids[i] = jid.String()
	}
	return fmt.Sprintf("%s: %s", ErrJobAlreadyRunning, strings.Join(ids, ", "))
}

func (e *JobLimitError) Unwrap() error { return ErrJobAlreadyRunning }

// The plan's per-hour and per-day run limits. A refusal is a UsageLimitError
// wrapping one of them.
var (
	ErrDebugRunsPerHourLimit            error = errors.New("this workspace's plan does not allow more debug runs this hour")
	ErrDeploymentRunsPerDayLimit        error = errors.New("this workspace's plan does not allow more deployment runs today")
	ErrTimeDrivenTriggerRunsPerDayLimit error = errors.New("this workspace's plan does not allow more time-driven trigger runs today")
)

// UsageLimitError refuses a run because its workspace has used up one of the
// plan's per-hour or per-day runs, with the time the count resets.
type UsageLimitError struct {
	ResetAt time.Time
	Err     error
}

func (e *UsageLimitError) Error() string { return e.Err.Error() }

func (e *UsageLimitError) Unwrap() error { return e.Err }

type Job interface {
	Cancel(context.Context, id.JobID) (*job.Job, error)
	Fetch(context.Context, []id.JobID) ([]*job.Job, error)
	FindByID(context.Context, id.JobID) (*job.Job, error)
	FindByWorkspace(context.Context, accountsid.WorkspaceID, *PaginationParam, *string) ([]*job.Job, *PageBasedInfo, error)
	GetStatus(context.Context, id.JobID) (job.Status, error)
	// Refresh brings a job's stored status up to date with the worker, as the
	// monitoring loop would, and returns the job as stored afterwards. It is
	// for jobs no monitoring loop is watching, such as ones left behind by a
	// server restart.
	Refresh(context.Context, *job.Job) (*job.Job, error)
	// RunCloudRunWorker runs j on the Cloud Run worker in the background.
	// notStarted, when set, is called if the worker is known not to have
	// started the run.
	RunCloudRunWorker(j *job.Job, p gateway.RunJobParam, notStarted func())
	PreviewSchemaCloudRunWorker(j *job.Job, p gateway.ProbeSchemaParam)
	StartMonitoring(context.Context, *job.Job, *string) error
	Subscribe(context.Context, id.JobID) (chan job.Status, error)
	Unsubscribe(id.JobID, chan job.Status)
}
