package postgres_test

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/projectAccess"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newProjectAccess(paid id.ProjectAccessID, pid id.ProjectID) *projectAccess.ProjectAccess {
	pa, err := projectAccess.New().
		ID(paid).
		Project(pid).
		Token("shr_testtoken").
		IsPublic(true).
		Build()
	if err != nil {
		panic(err)
	}
	return pa
}

func TestProjectAccess_Save_FindByProjectID(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	paid := id.NewProjectAccessID()
	pid := id.NewProjectID()
	pa := newProjectAccess(paid, pid)
	r := postgres.NewProjectAccess(pgxx.NewClient(pool))

	require.NoError(t, r.Save(ctx, pa))

	got, err := r.FindByProjectID(ctx, pid)
	require.NoError(t, err)
	require.NotNil(t, got)
	assert.Equal(t, paid, got.ID())
	assert.Equal(t, pid, got.Project())
	assert.Equal(t, "shr_testtoken", got.Token())
	assert.True(t, got.IsPublic())
}

func TestProjectAccess_Save_FindByToken(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	paid := id.NewProjectAccessID()
	pid := id.NewProjectID()
	pa := newProjectAccess(paid, pid)
	r := postgres.NewProjectAccess(pgxx.NewClient(pool))

	require.NoError(t, r.Save(ctx, pa))

	got, err := r.FindByToken(ctx, "shr_testtoken")
	require.NoError(t, err)
	require.NotNil(t, got)
	assert.Equal(t, paid, got.ID())
	assert.Equal(t, pid, got.Project())
}

func TestProjectAccess_FindByProjectID_NotFound(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewProjectAccess(pgxx.NewClient(pool))

	got, err := r.FindByProjectID(ctx, id.NewProjectID())
	assert.Nil(t, got)
	assert.ErrorIs(t, err, rerror.ErrNotFound)
}

func TestProjectAccess_FindByToken_NotFound(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewProjectAccess(pgxx.NewClient(pool))

	got, err := r.FindByToken(ctx, "shr_nonexistent")
	assert.Nil(t, got)
	assert.ErrorIs(t, err, rerror.ErrNotFound)
}

func TestProjectAccess_Save_Upsert(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	paid := id.NewProjectAccessID()
	pid := id.NewProjectID()
	pa := newProjectAccess(paid, pid)
	r := postgres.NewProjectAccess(pgxx.NewClient(pool))

	require.NoError(t, r.Save(ctx, pa))

	// Make private: token cleared, isPublic false.
	require.NoError(t, pa.MakePrivate())
	require.NoError(t, r.Save(ctx, pa))

	got, err := r.FindByProjectID(ctx, pid)
	require.NoError(t, err)
	assert.False(t, got.IsPublic())
	assert.Equal(t, "", got.Token())
}

func TestProjectAccess_Save_PrivateEntry(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	paid := id.NewProjectAccessID()
	pid := id.NewProjectID()
	// Private entry: token empty, isPublic false.
	pa, err := projectAccess.New().
		ID(paid).
		Project(pid).
		Token("").
		IsPublic(false).
		Build()
	require.NoError(t, err)
	r := postgres.NewProjectAccess(pgxx.NewClient(pool))

	require.NoError(t, r.Save(ctx, pa))

	got, err := r.FindByProjectID(ctx, pid)
	require.NoError(t, err)
	assert.False(t, got.IsPublic())
	assert.Equal(t, "", got.Token())
}
