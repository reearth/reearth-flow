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

// JobRunBy checks that a job keeps the user who started it.
func JobRunBy(t *testing.T, newRepo func(*testing.T) repo.Job) {
	t.Helper()
	ctx := context.Background()

	t.Run("keeps the user who started the job", func(t *testing.T) {
		t.Parallel()
		r := newRepo(t)
		uid := accountsid.NewUserID()
		j := job.New().NewID().Workspace(accountsid.NewWorkspaceID()).Status(job.StatusPending).
			StartedAt(time.Now()).RunBy(&uid).MustBuild()
		require.NoError(t, r.Save(ctx, j))

		got, err := r.FindByID(ctx, j.ID())

		require.NoError(t, err)
		require.NotNil(t, got.RunBy())
		assert.Equal(t, uid, *got.RunBy())
	})

	t.Run("a job no user started has no user", func(t *testing.T) {
		t.Parallel()
		r := newRepo(t)
		j := job.New().NewID().Workspace(accountsid.NewWorkspaceID()).Status(job.StatusPending).
			StartedAt(time.Now()).MustBuild()
		require.NoError(t, r.Save(ctx, j))

		got, err := r.FindByID(ctx, j.ID())

		require.NoError(t, err)
		assert.Nil(t, got.RunBy())
	})
}
