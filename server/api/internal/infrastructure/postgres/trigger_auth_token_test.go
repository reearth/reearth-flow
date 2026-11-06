package postgres_test

import (
	"context"
	"io/fs"
	"sort"
	"strings"
	"testing"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	flowdb "github.com/reearth/reearth-flow/db"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/pgxx/pgxtest"
	"github.com/samber/lo"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestTrigger_Save_StoresOnlyTokenHash(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	wid := accountsid.NewWorkspaceID()
	tr := trigger.New().NewID().Workspace(wid).Deployment(id.NewDeploymentID()).
		Description("hook").EventSource(trigger.EventSourceTypeAPIDriven).MustBuild()
	token := tr.GenerateAuthToken()
	r := postgres.NewTrigger(pgxx.NewClient(pool))
	require.NoError(t, r.Save(ctx, tr))

	var row string
	require.NoError(t, pool.QueryRow(ctx, `SELECT row_to_json(t)::text FROM triggers t WHERE id = $1`, tr.ID().String()).Scan(&row))
	assert.NotContains(t, row, strings.TrimPrefix(token, trigger.AuthTokenPrefix), "the token itself is never stored")

	got, err := r.FindByID(ctx, tr.ID())
	require.NoError(t, err)
	assert.True(t, got.VerifyAuthToken(token), "the saved trigger still accepts its token")
	assert.Equal(t, tr.AuthTokenHint(), got.AuthTokenHint())

	// Listing maps columns by position, so it is checked separately.
	page := &interfaces.PaginationParam{Page: &interfaces.PageBasedPaginationParam{Page: 1, PageSize: 10}}
	for _, p := range []*interfaces.PaginationParam{nil, page} {
		listed, _, err := r.FindByWorkspace(ctx, wid, p, nil)
		require.NoError(t, err)
		require.Len(t, listed, 1)
		assert.True(t, listed[0].VerifyAuthToken(token))
		assert.Equal(t, tr.AuthTokenHint(), listed[0].AuthTokenHint())
	}
}

// TestMigration_HashTriggerAuthTokens_ExistingRows applies the migration that
// hashes trigger tokens to a database that already holds plain-text tokens,
// which a fresh test database never exercises.
func TestMigration_HashTriggerAuthTokens_ExistingRows(t *testing.T) {
	pool := pgxtest.Connect(t)(t)
	ctx := context.Background()

	weak, long, timed, emptied := id.NewTriggerID(), id.NewTriggerID(), id.NewTriggerID(), id.NewTriggerID()
	migrateAround(t, ctx, pool, "_hash_trigger_auth_tokens.sql", func() {
		wid, did := accountsid.NewWorkspaceID().String(), id.NewDeploymentID().String()
		insert := func(tid id.TriggerID, eventSource string, authToken *string) {
			_, err := pool.Exec(ctx,
				`INSERT INTO triggers (id, workspace_id, deployment_id, description, event_source, auth_token, created_at, updated_at)
				 VALUES ($1, $2, $3, 'old', $4, $5, $6, $6)`,
				tid.String(), wid, did, eventSource, authToken, time.Now())
			require.NoError(t, err)
		}
		insert(weak, "API_DRIVEN", lo.ToPtr("abc"))
		insert(long, "API_DRIVEN", lo.ToPtr("a-much-longer-hand-picked-token"))
		insert(timed, "TIME_DRIVEN", nil)
		// Switching a trigger to time-driven used to leave an empty token.
		insert(emptied, "TIME_DRIVEN", lo.ToPtr(""))
	})

	var plainColumns int
	require.NoError(t, pool.QueryRow(ctx,
		`SELECT count(*) FROM information_schema.columns WHERE table_name = 'triggers' AND column_name = 'auth_token'`,
	).Scan(&plainColumns))
	assert.Zero(t, plainColumns, "no plain-text token is left")

	r := postgres.NewTrigger(pgxx.NewClient(pool))
	find := func(tid id.TriggerID) *trigger.Trigger {
		got, err := r.FindByID(ctx, tid)
		require.NoError(t, err)
		return got
	}

	got := find(weak)
	assert.True(t, got.VerifyAuthToken("abc"), "callers keep using the token they have")
	assert.False(t, got.VerifyAuthToken("abd"))
	assert.Nil(t, got.AuthTokenHint(), "a hint would give a short token away")

	assert.True(t, find(long).VerifyAuthToken("a-much-longer-hand-picked-token"))
	assert.Nil(t, find(timed).AuthTokenHash())
	assert.Nil(t, find(emptied).AuthTokenHash())
}

// migrateAround applies the migrations before the one named by suffix, runs
// seed, then applies the rest.
func migrateAround(t *testing.T, ctx context.Context, pool *pgxpool.Pool, suffix string, seed func()) {
	t.Helper()
	files, err := fs.Glob(flowdb.MigrationsFS, "migrations/*.sql")
	require.NoError(t, err)
	sort.Strings(files)
	idx := -1
	for i, f := range files {
		if strings.HasSuffix(f, suffix) {
			idx = i
		}
	}
	require.NotEqual(t, -1, idx, "no migration ends with %s", suffix)

	apply := func(f string) {
		b, err := fs.ReadFile(flowdb.MigrationsFS, f)
		require.NoError(t, err)
		_, err = pool.Exec(ctx, string(b))
		require.NoError(t, err, f)
	}
	for _, f := range files[:idx] {
		apply(f)
	}
	seed()
	for _, f := range files[idx:] {
		apply(f)
	}
}
