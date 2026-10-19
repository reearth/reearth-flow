package repotest

import (
	"context"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
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
