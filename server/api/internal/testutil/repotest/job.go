package repotest

import (
	"context"
	"fmt"
	"sync"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// JobCountRunning checks repo.Job.CountRunningByWorkspace and
// FindRunningByWorkspace: only the workspace's pending and running debug runs
// started at or after the cutoff count, and preview-schema probes never do.
func JobCountRunning(t *testing.T, r repo.Job) {
	t.Helper()
	ctx := context.Background()
	ws, other := accountsid.NewWorkspaceID(), accountsid.NewWorkspaceID()
	// Whole seconds, so the cutoff compares the same in every backend.
	since := time.Now().UTC().Truncate(time.Second).Add(-time.Hour)

	save := func(ws accountsid.WorkspaceID, status job.Status, startedAt time.Time, mode job.Mode, debug *bool) {
		t.Helper()
		b := job.New().NewID().Workspace(ws).Status(status).StartedAt(startedAt).Debug(debug)
		if mode != "" {
			b = b.Mode(mode)
		}
		require.NoError(t, r.Save(ctx, b.MustBuild()))
	}
	yes, no := true, false

	count := func() int {
		t.Helper()
		n, err := r.CountRunningByWorkspace(ctx, ws, since)
		require.NoError(t, err)
		return n
	}

	assert.Zero(t, count(), "no jobs yet")

	save(ws, job.StatusPending, since.Add(time.Minute), "", &no)                     // deployed run, pending: not a debug run
	save(ws, job.StatusRunning, since, job.ModeRun, &yes)                            // debug run, started exactly at the cutoff
	save(ws, job.StatusPending, since.Add(2*time.Minute), "", &yes)                  // debug run, legacy doc without a mode
	save(ws, job.StatusRunning, since.Add(time.Minute), "", nil)                     // trigger run, no debug flag
	save(ws, job.StatusCompleted, since.Add(time.Minute), "", &no)                   // finished
	save(ws, job.StatusFailed, since.Add(time.Minute), "", &no)                      // finished
	save(ws, job.StatusCancelled, since.Add(time.Minute), "", &no)                   // finished
	save(ws, job.StatusRunning, since.Add(-time.Second), "", &no)                    // stuck: started before the cutoff
	save(ws, job.StatusRunning, since.Add(time.Minute), job.ModePreviewSchema, &yes) // probe
	save(other, job.StatusRunning, since.Add(time.Minute), "", &no)                  // another workspace

	assert.Equal(t, 2, count())

	found, err := r.FindRunningByWorkspace(ctx, ws, since)
	require.NoError(t, err)
	require.Len(t, found, 2, "finds the jobs it counts")
	for i := 1; i < len(found); i++ {
		assert.False(t, found[i].StartedAt().Before(found[i-1].StartedAt()), "oldest first")
	}
}

// JobSaveIfRunningBelow checks repo.Job.SaveIfRunningBelow: it counts like
// CountRunningByWorkspace, saves only below the limit, and concurrent calls
// never admit more jobs than the limit between them.
func JobSaveIfRunningBelow(t *testing.T, r repo.Job) {
	t.Helper()
	ctx := context.Background()
	since := time.Now().UTC().Truncate(time.Second).Add(-time.Hour)
	newJob := func(ws accountsid.WorkspaceID, startedAt time.Time) *job.Job {
		debug := true
		return job.New().NewID().Workspace(ws).Status(job.StatusPending).StartedAt(startedAt).Debug(&debug).MustBuild()
	}
	count := func(ws accountsid.WorkspaceID) int {
		t.Helper()
		n, err := r.CountRunningByWorkspace(ctx, ws, since)
		require.NoError(t, err)
		return n
	}

	t.Run("saves below the limit and refuses at it", func(t *testing.T) {
		ws := accountsid.NewWorkspaceID()
		require.NoError(t, r.Save(ctx, newJob(ws, since.Add(-time.Minute)))) // stale, not counted

		first := newJob(ws, since.Add(time.Minute))
		require.NoError(t, r.SaveIfRunningBelow(ctx, first, 1, since))
		got, err := r.FindByID(ctx, first.ID())
		require.NoError(t, err)
		assert.Equal(t, first.ID(), got.ID())

		second := newJob(ws, since.Add(time.Minute))
		assert.ErrorIs(t, r.SaveIfRunningBelow(ctx, second, 1, since), interfaces.ErrJobAlreadyRunning)
		_, err = r.FindByID(ctx, second.ID())
		assert.Error(t, err, "a refused job is not saved")

		assert.NoError(t, r.SaveIfRunningBelow(ctx, second, 2, since), "a higher limit admits it")
	})

	for _, limit := range []int{1, 2} {
		t.Run(fmt.Sprintf("concurrent calls admit at most %d", limit), func(t *testing.T) {
			ws := accountsid.NewWorkspaceID()
			const calls = 16
			var wg sync.WaitGroup
			errs := make([]error, calls)
			start := make(chan struct{})
			for i := range calls {
				wg.Add(1)
				go func() {
					defer wg.Done()
					<-start
					errs[i] = r.SaveIfRunningBelow(ctx, newJob(ws, time.Now()), limit, since)
				}()
			}
			close(start)
			wg.Wait()

			admitted := 0
			for _, err := range errs {
				if err == nil {
					admitted++
					continue
				}
				assert.ErrorIs(t, err, interfaces.ErrJobAlreadyRunning)
			}
			assert.Equal(t, limit, admitted)
			assert.Equal(t, limit, count(ws), "refused jobs are not left behind")
		})
	}
}

// JobSaveIfRunningBelowInTx is JobSaveIfRunningBelow with every call inside a
// transaction. The run entry points do not make one, but a caller that does
// must still never admit too many, and a call that loses a race has already
// saved its job, so the refusal must roll that job back.
func JobSaveIfRunningBelowInTx(t *testing.T, r repo.Job, tr usecasex.Transactor) {
	t.Helper()
	ctx := context.Background()
	since := time.Now().UTC().Truncate(time.Second).Add(-time.Hour)
	newJob := func(ws accountsid.WorkspaceID) *job.Job {
		debug := true
		return job.New().NewID().Workspace(ws).Status(job.StatusPending).StartedAt(time.Now()).Debug(&debug).MustBuild()
	}
	saveInTx := func(j *job.Job, limit int) error {
		return tr.WithinTransaction(ctx, func(ctx context.Context) error {
			return r.SaveIfRunningBelow(ctx, j, limit, since)
		})
	}

	t.Run("saves below the limit and refuses at it", func(t *testing.T) {
		ws := accountsid.NewWorkspaceID()
		first, second := newJob(ws), newJob(ws)

		require.NoError(t, saveInTx(first, 1))
		_, err := r.FindByID(ctx, first.ID())
		assert.NoError(t, err, "the admitted job is committed")

		assert.ErrorIs(t, saveInTx(second, 1), interfaces.ErrJobAlreadyRunning)
		_, err = r.FindByID(ctx, second.ID())
		assert.Error(t, err, "a refused job is not saved")
	})

	for _, limit := range []int{1, 2} {
		t.Run(fmt.Sprintf("concurrent transactions admit at most %d and roll back the rest", limit), func(t *testing.T) {
			ws := accountsid.NewWorkspaceID()
			const calls = 16
			var wg sync.WaitGroup
			jobs := make([]*job.Job, calls)
			errs := make([]error, calls)
			start := make(chan struct{})
			for i := range calls {
				jobs[i] = newJob(ws)
				wg.Add(1)
				go func() {
					defer wg.Done()
					<-start
					errs[i] = saveInTx(jobs[i], limit)
				}()
			}
			close(start)
			wg.Wait()

			admitted := 0
			for i, err := range errs {
				_, findErr := r.FindByID(ctx, jobs[i].ID())
				if err == nil {
					admitted++
					assert.NoError(t, findErr, "an admitted job is committed")
					continue
				}
				assert.ErrorIs(t, err, interfaces.ErrJobAlreadyRunning)
				assert.Error(t, findErr, "a refused job is rolled back")
			}
			// A transaction that loses the race cannot count again inside the
			// aborted transaction, so it is refused even if a slot is left:
			// contention may admit fewer than limit, never more. This is why
			// the run entry points save outside a transaction.
			assert.GreaterOrEqual(t, admitted, 1)
			assert.LessOrEqual(t, admitted, limit)
			n, err := r.CountRunningByWorkspace(ctx, ws, since)
			require.NoError(t, err)
			assert.Equal(t, admitted, n, "only admitted jobs are left")
		})
	}
}

// JobScheduledRun checks that a time-driven trigger has at most one job for
// each scheduled time: FindByScheduledRun finds it, and Save and
// SaveIfRunningBelow refuse a second job for the same time. Each subtest gets
// its own repository from newRepo, because a memory repository keeps the jobs
// it is given and a subtest changes its own.
func JobScheduledRun(t *testing.T, newRepo func(*testing.T) repo.Job) {
	t.Helper()
	ctx := context.Background()
	at := time.Date(2026, 11, 6, 0, 0, 0, 0, time.UTC)
	newJob := func(ws accountsid.WorkspaceID, run *job.ScheduledRun) *job.Job {
		return job.New().NewID().Workspace(ws).Status(job.StatusPending).StartedAt(time.Now()).ScheduledRun(run).MustBuild()
	}

	t.Run("finds the job for a scheduled time", func(t *testing.T) {
		t.Parallel()
		r := newRepo(t)

		ws := accountsid.NewWorkspaceID()
		run := job.ScheduledRun{Trigger: id.NewTriggerID(), At: at}
		j := newJob(ws, &run)
		require.NoError(t, r.Save(ctx, j))

		got, err := r.FindByScheduledRun(ctx, run)
		require.NoError(t, err)
		assert.Equal(t, j.ID(), got.ID())
		require.NotNil(t, got.ScheduledRun())
		assert.Equal(t, run.Trigger, got.ScheduledRun().Trigger)
		assert.True(t, run.At.Equal(got.ScheduledRun().At))

		_, err = r.FindByScheduledRun(ctx, job.ScheduledRun{Trigger: run.Trigger, At: at.Add(24 * time.Hour)})
		assert.ErrorIs(t, err, rerror.ErrNotFound, "another time of the trigger")
		_, err = r.FindByScheduledRun(ctx, job.ScheduledRun{Trigger: id.NewTriggerID(), At: at})
		assert.ErrorIs(t, err, rerror.ErrNotFound, "another trigger at the time")
	})

	t.Run("refuses a second job for the same scheduled time", func(t *testing.T) {
		t.Parallel()
		r := newRepo(t)

		ws := accountsid.NewWorkspaceID()
		run := job.ScheduledRun{Trigger: id.NewTriggerID(), At: at}
		first := newJob(ws, &run)
		require.NoError(t, r.Save(ctx, first))
		require.NoError(t, r.Save(ctx, first), "saving the same job again is not a second job")

		second := newJob(ws, &run)
		assert.ErrorIs(t, r.Save(ctx, second), interfaces.ErrScheduledRunInProgress)
		assert.ErrorIs(t, r.SaveIfRunningBelow(ctx, second, 10, at.Add(-time.Hour)), interfaces.ErrScheduledRunInProgress)
		_, err := r.FindByID(ctx, second.ID())
		assert.Error(t, err, "the second job is not saved")
	})

	t.Run("a job that gives up its scheduled time frees it", func(t *testing.T) {
		t.Parallel()
		r := newRepo(t)

		ws := accountsid.NewWorkspaceID()
		run := job.ScheduledRun{Trigger: id.NewTriggerID(), At: at}
		first := newJob(ws, &run)
		require.NoError(t, r.Save(ctx, first))
		first.SetScheduledRun(nil)
		require.NoError(t, r.Save(ctx, first))

		second := newJob(ws, &run)
		require.NoError(t, r.Save(ctx, second))
		got, err := r.FindByScheduledRun(ctx, run)
		require.NoError(t, err)
		assert.Equal(t, second.ID(), got.ID())
	})

	t.Run("jobs not started by a schedule never conflict", func(t *testing.T) {
		t.Parallel()
		r := newRepo(t)

		ws := accountsid.NewWorkspaceID()
		require.NoError(t, r.Save(ctx, newJob(ws, nil)))
		require.NoError(t, r.Save(ctx, newJob(ws, nil)))
	})
}
