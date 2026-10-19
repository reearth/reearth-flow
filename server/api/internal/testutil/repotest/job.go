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
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// JobCountRunning checks repo.Job.CountRunningByWorkspace: only the workspace's
// pending and running jobs started at or after the cutoff count, and
// preview-schema probes never do.
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

	save(ws, job.StatusPending, since.Add(time.Minute), "", &no)                     // deployed run, pending
	save(ws, job.StatusRunning, since, job.ModeRun, &yes)                            // debug run, started exactly at the cutoff
	save(ws, job.StatusRunning, since.Add(time.Minute), "", nil)                     // trigger run, no debug flag
	save(ws, job.StatusCompleted, since.Add(time.Minute), "", &no)                   // finished
	save(ws, job.StatusFailed, since.Add(time.Minute), "", &no)                      // finished
	save(ws, job.StatusCancelled, since.Add(time.Minute), "", &no)                   // finished
	save(ws, job.StatusRunning, since.Add(-time.Second), "", &no)                    // stuck: started before the cutoff
	save(ws, job.StatusRunning, since.Add(time.Minute), job.ModePreviewSchema, &yes) // probe
	save(other, job.StatusRunning, since.Add(time.Minute), "", &no)                  // another workspace

	assert.Equal(t, 3, count())
}

// JobSaveIfRunningBelow checks repo.Job.SaveIfRunningBelow: it counts like
// CountRunningByWorkspace, saves only below the limit, and concurrent calls
// never admit more jobs than the limit between them.
func JobSaveIfRunningBelow(t *testing.T, r repo.Job) {
	t.Helper()
	ctx := context.Background()
	since := time.Now().UTC().Truncate(time.Second).Add(-time.Hour)
	newJob := func(ws accountsid.WorkspaceID, startedAt time.Time) *job.Job {
		return job.New().NewID().Workspace(ws).Status(job.StatusPending).StartedAt(startedAt).MustBuild()
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
