package repotest

import (
	"context"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// JobSaveIfNotTerminal checks a finished job is written once and later or concurrent writers lose.
func JobSaveIfNotTerminal(t *testing.T, newRepo func(*testing.T) repo.Job) {
	t.Helper()
	ctx := context.Background()
	ws := accountsid.NewWorkspaceID()
	// Whole seconds, so times compare the same in every backend.
	startedAt := time.Now().UTC().Truncate(time.Second)

	// copyOf is a separate copy, as another server instance would load it.
	copyOf := func(jid id.JobID, status job.Status) *job.Job {
		return job.New().ID(jid).Workspace(ws).Status(status).StartedAt(startedAt).MustBuild()
	}

	t.Run("saves a running job", func(t *testing.T) {
		t.Parallel()
		r := newRepo(t)
		jid := id.NewJobID()
		require.NoError(t, r.Save(ctx, copyOf(jid, job.StatusPending)))

		saved, err := r.SaveIfNotTerminal(ctx, copyOf(jid, job.StatusRunning))

		require.NoError(t, err)
		assert.True(t, saved)
		got, err := r.FindByID(ctx, jid)
		require.NoError(t, err)
		assert.Equal(t, job.StatusRunning, got.Status())
	})

	t.Run("only the first writer finishes a job", func(t *testing.T) {
		t.Parallel()
		r := newRepo(t)
		jid := id.NewJobID()
		require.NoError(t, r.Save(ctx, copyOf(jid, job.StatusRunning)))

		first := copyOf(jid, job.StatusCompleted)
		completedAt := startedAt.Add(time.Minute)
		first.SetCompletedAt(&completedAt)
		saved, err := r.SaveIfNotTerminal(ctx, first)
		require.NoError(t, err)
		assert.True(t, saved)

		saved, err = r.SaveIfNotTerminal(ctx, copyOf(jid, job.StatusFailed))
		require.NoError(t, err)
		assert.False(t, saved, "a second terminal write")

		saved, err = r.SaveIfNotTerminal(ctx, copyOf(jid, job.StatusRunning))
		require.NoError(t, err)
		assert.False(t, saved, "a stale running write")

		got, err := r.FindByID(ctx, jid)
		require.NoError(t, err)
		assert.Equal(t, job.StatusCompleted, got.Status())
		require.NotNil(t, got.CompletedAt())
		assert.True(t, completedAt.Equal(*got.CompletedAt()))
	})

	t.Run("concurrent writers finish a job once", func(t *testing.T) {
		t.Parallel()
		r := newRepo(t)
		jid := id.NewJobID()
		require.NoError(t, r.Save(ctx, copyOf(jid, job.StatusRunning)))

		const writers = 8
		var won atomic.Int32
		var wg sync.WaitGroup
		for range writers {
			j := copyOf(jid, job.StatusCompleted)
			wg.Go(func() {
				saved, err := r.SaveIfNotTerminal(ctx, j)
				assert.NoError(t, err)
				if saved {
					won.Add(1)
				}
			})
		}
		wg.Wait()

		assert.Equal(t, int32(1), won.Load())
	})

	t.Run("keeps the webhook URL and when it was sent", func(t *testing.T) {
		t.Parallel()
		r := newRepo(t)
		jid := id.NewJobID()
		url := "https://example.com/hook"
		j := copyOf(jid, job.StatusRunning)
		j.SetNotificationURL(&url)
		require.NoError(t, r.Save(ctx, j))

		got, err := r.FindByID(ctx, jid)
		require.NoError(t, err)
		require.NotNil(t, got.NotificationURL())
		assert.Equal(t, url, *got.NotificationURL())
		assert.Nil(t, got.NotifiedAt())

		done := copyOf(jid, job.StatusCompleted)
		done.SetNotificationURL(&url)
		notifiedAt := startedAt.Add(time.Minute)
		done.SetNotifiedAt(&notifiedAt)
		require.NoError(t, r.Save(ctx, done))

		got, err = r.FindByID(ctx, jid)
		require.NoError(t, err)
		require.NotNil(t, got.NotifiedAt())
		assert.True(t, notifiedAt.Equal(*got.NotifiedAt()))
	})
}
