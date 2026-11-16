package interactor

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/google/uuid"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearth-flow/api/pkg/variable"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// A scheduled run that can never succeed returns an error the webhook can tell
// apart, so it answers without making the scheduler retry.
func TestTrigger_ExecuteTimeDrivenTrigger_PermanentErrors(t *testing.T) {
	t.Parallel()

	t.Run("a disabled trigger", func(t *testing.T) {
		t.Parallel()

		f := newParamFixture(t)
		tr := f.trigger(t, f.deployment(t))
		tr.SetEnabled(false)
		require.NoError(t, f.repo.Trigger.Save(f.ctx, tr))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID()})

		assert.ErrorIs(t, err, interfaces.ErrTriggerDisabled)
		assert.Empty(t, f.batch.submitted)
	})

	t.Run("a trigger that is not time-driven", func(t *testing.T) {
		t.Parallel()

		f := newParamFixture(t)
		tr := f.trigger(t, f.deployment(t))
		tr.SetEventSource("API_DRIVEN")
		tr.SetTimeInterval("")
		require.NoError(t, f.repo.Trigger.Save(f.ctx, tr))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID()})

		assert.ErrorIs(t, err, interfaces.ErrTriggerNotTimeDriven)
		assert.Empty(t, f.batch.submitted)
	})

	t.Run("a trigger value that does not fit its variable's type", func(t *testing.T) {
		t.Parallel()

		f := newParamFixture(t)
		d := f.deployment(t, f.param(t, "area", 0, false, "deployed"))
		tr := f.trigger(t, d, variable.Variable{Key: "area", Type: parameter.TypeNumber, Value: "not a number"})
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID()})

		assert.ErrorIs(t, err, interfaces.ErrInvalidVariableValue)
		assert.Empty(t, f.batch.submitted)
	})
}

// heldRunBatch refuses every submit, and records the job that held the
// scheduled run when the submit was made.
type heldRunBatch struct {
	gateway.Batch
	jobs repo.Job
	held *job.Job
	run  job.ScheduledRun
}

func (b *heldRunBatch) SubmitJob(ctx context.Context, _ id.JobID, _ string, _ string, _ map[string]string, _ id.ProjectID, _ accountsid.WorkspaceID, _ *id.JobID, _ *uuid.UUID, _ time.Duration) (string, error) {
	b.held, _ = b.jobs.FindByScheduledRun(ctx, b.run)
	return "", errSubmitRefused
}

// The scheduler sends the same scheduled time with every retry of one run, so
// a retry never starts a second job for it.
func TestTrigger_ExecuteTimeDrivenTrigger_OneJobPerScheduledTime(t *testing.T) {
	t.Parallel()

	at := time.Date(2026, 11, 6, 0, 0, 0, 0, time.UTC)

	t.Run("a retry of a run already submitted returns its job, even with the workspace full", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		tr := f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))
		did := tr.Deployment()
		started := job.New().NewID().Workspace(f.ws).Deployment(&did).Status(job.StatusRunning).
			GCPJobID("gcp-1").StartedAt(at).ScheduledRun(&job.ScheduledRun{Trigger: tr.ID(), At: at}).MustBuild()
		require.NoError(t, f.repo.Job.Save(f.ctx, started))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		got, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID(), ScheduledAt: &at})

		require.NoError(t, err)
		assert.Equal(t, started.ID(), got.ID())
	})

	t.Run("a retry while the first attempt is still starting its job", func(t *testing.T) {
		t.Parallel()

		f := newParamFixture(t)
		tr := f.trigger(t, f.deployment(t))
		did := tr.Deployment()
		starting := job.New().NewID().Workspace(f.ws).Deployment(&did).Status(job.StatusPending).
			StartedAt(at).ScheduledRun(&job.ScheduledRun{Trigger: tr.ID(), At: at}).MustBuild()
		require.NoError(t, f.repo.Job.Save(f.ctx, starting))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID(), ScheduledAt: &at})

		assert.ErrorIs(t, err, interfaces.ErrScheduledRunInProgress)
		assert.Empty(t, f.batch.submitted)
	})

	t.Run("the new job holds the scheduled time while it is submitted", func(t *testing.T) {
		t.Parallel()

		f := newParamFixture(t)
		tr := f.trigger(t, f.deployment(t))
		b := &heldRunBatch{jobs: f.repo.Job, run: job.ScheduledRun{Trigger: tr.ID(), At: at}}
		f.gw.Batch = b
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID(), ScheduledAt: &at})

		assert.ErrorIs(t, err, interfaces.ErrJobCreationFailed)
		assert.NotNil(t, b.held, "no job held the scheduled time at submit")
	})

	t.Run("a retry after the submit failed starts the run again", func(t *testing.T) {
		t.Parallel()

		f := newParamFixture(t)
		tr := f.trigger(t, f.deployment(t))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)
		param := interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID(), ScheduledAt: &at}

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, param)
		require.ErrorIs(t, err, interfaces.ErrJobCreationFailed)
		_, err = i.ExecuteTimeDrivenTrigger(f.ctx, param)

		assert.ErrorIs(t, err, interfaces.ErrJobCreationFailed)
		assert.Len(t, f.batch.submitted, 2)
	})
}

// submittingBatch accepts every submit.
type submittingBatch struct{ gateway.Batch }

func (submittingBatch) SubmitJob(context.Context, id.JobID, string, string, map[string]string, id.ProjectID, accountsid.WorkspaceID, *id.JobID, *uuid.UUID, time.Duration) (string, error) {
	return "gcp-1", nil
}

// unmonitoredJobs starts no monitoring loop, which would poll a real backend.
type unmonitoredJobs struct{ interfaces.Job }

func (unmonitoredJobs) StartMonitoring(context.Context, *job.Job, *string) error { return nil }

// A refused scheduled run is recorded on its trigger, so its user can see why
// it did not run, and whether it is still being retried.
func TestTrigger_ExecuteTimeDrivenTrigger_RecordsRefusals(t *testing.T) {
	t.Parallel()

	at := time.Date(2026, 11, 6, 0, 0, 0, 0, time.UTC)
	refusal := func(t *testing.T, r *repo.Container, ctx context.Context, tid id.TriggerID) *trigger.Refusal {
		t.Helper()
		got, err := r.Trigger.FindByID(ctx, tid)
		require.NoError(t, err)
		return got.LastRefusal()
	}

	t.Run("a skipped run records why, and no retry", func(t *testing.T) {
		t.Parallel()

		f := newParamFixture(t)
		d := f.deployment(t, f.param(t, "area", 0, false, "deployed"))
		tr := f.trigger(t, d, variable.Variable{Key: "area", Type: parameter.TypeNumber, Value: "not a number"})
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID(), ScheduledAt: &at})
		require.Error(t, err)

		got := refusal(t, f.repo, f.ctx, tr.ID())
		require.NotNil(t, got)
		assert.Equal(t, trigger.RefusalInvalidVariables, got.Code)
		assert.Nil(t, got.RetryUntil)
	})

	t.Run("a run over the day's limit records a skip", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		f.gw.UsageCounter = usedUpCounter{}
		tr := f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID(), ScheduledAt: &at})
		require.ErrorIs(t, err, interfaces.ErrTimeDrivenTriggerRunsPerDayLimit)

		got := refusal(t, f.repo, f.ctx, tr.ID())
		require.NotNil(t, got)
		assert.Equal(t, trigger.RefusalTimeDrivenTriggerRunsDailyLimit, got.Code)
		assert.Nil(t, got.RetryUntil)
	})

	t.Run("a submit that fails records a server error, retried", func(t *testing.T) {
		t.Parallel()

		f := newParamFixture(t)
		tr := f.trigger(t, f.deployment(t))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID(), ScheduledAt: &at})
		require.ErrorIs(t, err, interfaces.ErrJobCreationFailed)

		got := refusal(t, f.repo, f.ctx, tr.ID())
		require.NotNil(t, got)
		assert.Equal(t, trigger.RefusalServerError, got.Code)
		assert.NotNil(t, got.RetryUntil)
	})

	t.Run("a run that starts clears the refusal", func(t *testing.T) {
		t.Parallel()

		f := newParamFixture(t)
		f.gw.Batch = submittingBatch{}
		tr := f.trigger(t, f.deployment(t))
		tr.SetLastRefusal(&trigger.Refusal{At: at, Code: trigger.RefusalTimeDrivenTriggerRunsDailyLimit})
		require.NoError(t, f.repo.Trigger.Save(f.ctx, tr))
		i := NewTrigger(f.repo, f.gw, unmonitoredJobs{f.job}, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID(), ScheduledAt: &at})
		require.NoError(t, err)

		assert.Nil(t, refusal(t, f.repo, f.ctx, tr.ID()))
	})

	t.Run("a retry while the run is starting records nothing", func(t *testing.T) {
		t.Parallel()

		f := newParamFixture(t)
		tr := f.trigger(t, f.deployment(t))
		did := tr.Deployment()
		starting := job.New().NewID().Workspace(f.ws).Deployment(&did).Status(job.StatusPending).
			StartedAt(at).ScheduledRun(&job.ScheduledRun{Trigger: tr.ID(), At: at}).MustBuild()
		require.NoError(t, f.repo.Job.Save(f.ctx, starting))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID(), ScheduledAt: &at})
		require.ErrorIs(t, err, interfaces.ErrScheduledRunInProgress)

		assert.Nil(t, refusal(t, f.repo, f.ctx, tr.ID()))
	})

	// Cloud Scheduler always sends the scheduled time, so a call without it
	// is not a scheduled run, and nothing retries it.
	t.Run("a call with no scheduled time records nothing", func(t *testing.T) {
		t.Parallel()

		f := newPlanFixture(t)
		tr := f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID()})
		require.ErrorIs(t, err, errUploadFailed)

		assert.Nil(t, refusal(t, f.repo, f.ctx, tr.ID()))
	})
}

// failingScheduler fails to update a trigger's scheduler job, as when Cloud
// Scheduler cannot resume the job of a trigger enabled again.
type failingScheduler struct{ gateway.Scheduler }

func (failingScheduler) UpdateScheduledJob(context.Context, *trigger.Trigger) error {
	return errors.New("failed to resume scheduled job")
}

// An edit whose scheduler job cannot be brought in line is still saved, and
// says so, so its user can save it again. Cloud Scheduler's own error is only
// logged.
func TestTrigger_Update_KeepsTheEditWhenTheSchedulerJobCannotBeUpdated(t *testing.T) {
	t.Parallel()

	f := newParamFixture(t)
	f.gw.Scheduler = failingScheduler{}
	tr := f.trigger(t, f.deployment(t))
	i := NewTrigger(f.repo, f.gw, f.job, f.perm)

	desc := "edited"
	_, _, err := i.Update(f.ctx, interfaces.UpdateTriggerParam{ID: tr.ID(), Description: &desc})
	require.ErrorIs(t, err, interfaces.ErrTriggerScheduleNotUpdated)
	assert.NotContains(t, err.Error(), "resume")

	got, err := f.repo.Trigger.FindByID(f.ctx, tr.ID())
	require.NoError(t, err)
	assert.Equal(t, "edited", got.Description())
}

// trackingTransactor knows whether one of its transactions is open.
type trackingTransactor struct {
	usecasex.Transactor
	open bool
}

func (t *trackingTransactor) WithinTransaction(ctx context.Context, fn func(context.Context) error) error {
	t.open = true
	defer func() { t.open = false }()
	return t.Transactor.WithinTransaction(ctx, fn)
}

type schedulerCall struct {
	op            string
	inTransaction bool
}

// recordingScheduler records each call made to it, and whether a transaction
// was open at the time.
type recordingScheduler struct {
	gateway.Scheduler
	tx    *trackingTransactor
	calls []schedulerCall
}

func (s *recordingScheduler) record(op string) error {
	s.calls = append(s.calls, schedulerCall{op: op, inTransaction: s.tx.open})
	return nil
}

func (s *recordingScheduler) CreateScheduledJob(context.Context, *trigger.Trigger) error {
	return s.record("create")
}

func (s *recordingScheduler) UpdateScheduledJob(context.Context, *trigger.Trigger) error {
	return s.record("update")
}

func (s *recordingScheduler) DeleteScheduledJob(context.Context, id.TriggerID) error {
	return s.record("delete")
}

// Cloud Scheduler is called only once an edit is saved, so a call that fails,
// or whose answer is lost after Cloud Scheduler applied it, cannot roll back
// the edit and leave the job on a schedule its trigger does not have. A
// trigger made time-driven has its job updated too, which creates it, so a
// job left behind when it stopped being time-driven is brought in line rather
// than refusing the edit.
func TestTrigger_Update_CallsTheSchedulerAfterSaving(t *testing.T) {
	t.Parallel()

	tests := []struct {
		name string
		from trigger.EventSourceType
		to   trigger.EventSourceType
		want []schedulerCall
	}{
		{name: "a time-driven trigger edited", from: "TIME_DRIVEN", to: "TIME_DRIVEN", want: []schedulerCall{{op: "update"}}},
		{name: "a trigger made time-driven", from: "API_DRIVEN", to: "TIME_DRIVEN", want: []schedulerCall{{op: "update"}}},
		{name: "a trigger no longer time-driven", from: "TIME_DRIVEN", to: "API_DRIVEN", want: []schedulerCall{{op: "delete"}}},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()

			f := newParamFixture(t)
			tx := &trackingTransactor{Transactor: f.repo.Transaction}
			f.repo.Transaction = tx
			s := &recordingScheduler{tx: tx}
			f.gw.Scheduler = s
			tr := f.trigger(t, f.deployment(t))
			if tc.from != "TIME_DRIVEN" {
				tr.SetEventSource(tc.from)
				tr.SetTimeInterval("")
				require.NoError(t, f.repo.Trigger.Save(f.ctx, tr))
			}
			i := NewTrigger(f.repo, f.gw, f.job, f.perm)

			param := interfaces.UpdateTriggerParam{ID: tr.ID(), EventSource: tc.to}
			if tc.to == "TIME_DRIVEN" {
				param.TimeInterval = trigger.TimeIntervalEveryDay
			}
			_, _, err := i.Update(f.ctx, param)
			require.NoError(t, err)
			assert.Equal(t, tc.want, s.calls)
		})
	}
}

// changingBatch runs change while a job is being submitted, as another request
// would, and then refuses the submit.
type changingBatch struct {
	gateway.Batch
	change func()
}

func (b changingBatch) SubmitJob(context.Context, id.JobID, string, string, map[string]string, id.ProjectID, accountsid.WorkspaceID, *id.JobID, *uuid.UUID, time.Duration) (string, error) {
	b.change()
	return "", errSubmitRefused
}

// Recording a refusal changes only the refusal, so it neither undoes an edit
// nor brings back a trigger removed while the run was being started.
func TestTrigger_ExecuteTimeDrivenTrigger_RefusalKeepsChangesMadeMeanwhile(t *testing.T) {
	t.Parallel()

	at := time.Date(2026, 11, 6, 0, 0, 0, 0, time.UTC)

	t.Run("an edit", func(t *testing.T) {
		t.Parallel()

		f := newParamFixture(t)
		tr := f.trigger(t, f.deployment(t))
		f.gw.Batch = changingBatch{change: func() {
			edited := trigger.New().ID(tr.ID()).Workspace(tr.Workspace()).Deployment(tr.Deployment()).
				Description("edited").EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay).
				Enabled(false).CreatedAt(time.Now()).UpdatedAt(time.Now()).MustBuild()
			require.NoError(t, f.repo.Trigger.Save(f.ctx, edited))
		}}
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID(), ScheduledAt: &at})
		require.ErrorIs(t, err, interfaces.ErrJobCreationFailed)

		got, err := f.repo.Trigger.FindByID(f.ctx, tr.ID())
		require.NoError(t, err)
		assert.Equal(t, "edited", got.Description())
		assert.False(t, got.Enabled())
		require.NotNil(t, got.LastRefusal())
		assert.Equal(t, trigger.RefusalServerError, got.LastRefusal().Code)
	})

	t.Run("a removal", func(t *testing.T) {
		t.Parallel()

		f := newParamFixture(t)
		tr := f.trigger(t, f.deployment(t))
		f.gw.Batch = changingBatch{change: func() {
			require.NoError(t, f.repo.Trigger.Remove(f.ctx, tr.ID()))
		}}
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID(), ScheduledAt: &at})
		require.ErrorIs(t, err, interfaces.ErrJobCreationFailed)

		_, err = f.repo.Trigger.FindByID(f.ctx, tr.ID())
		assert.ErrorIs(t, err, rerror.ErrNotFound)
	})
}
