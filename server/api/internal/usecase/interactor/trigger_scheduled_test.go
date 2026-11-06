package interactor

import (
	"context"
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

func (b *heldRunBatch) SubmitJob(ctx context.Context, _ id.JobID, _ string, _ string, _ map[string]string, _ id.ProjectID, _ accountsid.WorkspaceID, _ *id.JobID, _ *uuid.UUID) (string, error) {
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
