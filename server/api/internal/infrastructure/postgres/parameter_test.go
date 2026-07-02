package postgres_test

import (
	"context"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newParam(pid id.ProjectID, idx int) *parameter.Parameter {
	p, err := parameter.New().
		ID(id.NewParameterID()).
		ProjectID(pid).
		Name("param").
		Type(parameter.TypeText).
		Index(idx).
		Required(false).
		Public(true).
		CreatedAt(time.Now()).
		UpdatedAt(time.Now()).
		Build()
	if err != nil {
		panic(err)
	}
	return p
}

func TestParameter_Save_FindByID(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	pid := id.NewProjectID()
	p := newParam(pid, 0)
	r := postgres.NewParameter(pgxx.NewClient(pool))
	require.NoError(t, r.Save(ctx, p))
	got, err := r.FindByID(ctx, p.ID())
	require.NoError(t, err)
	assert.Equal(t, p.ID(), got.ID())
	assert.Equal(t, pid, got.ProjectID())
	assert.Equal(t, "param", got.Name())
	assert.Equal(t, parameter.TypeText, got.Type())
	assert.Equal(t, 0, got.Index())
	assert.True(t, got.Public())
}

func TestParameter_FindByID_NotFound(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	_, err := postgres.NewParameter(pgxx.NewClient(pool)).FindByID(context.Background(), id.NewParameterID())
	assert.ErrorIs(t, err, rerror.ErrNotFound)
}

func TestParameter_FindByIDs_Order(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewParameter(pgxx.NewClient(pool))
	pid := id.NewProjectID()
	p1 := newParam(pid, 0)
	p2 := newParam(pid, 1)
	require.NoError(t, r.Save(ctx, p1))
	require.NoError(t, r.Save(ctx, p2))
	missing := id.NewParameterID()
	// request in reverse order with a missing entry
	got, err := r.FindByIDs(ctx, id.ParameterIDList{p2.ID(), missing, p1.ID()})
	require.NoError(t, err)
	require.NotNil(t, got)
	list := *got
	require.Len(t, list, 2) // missing id omitted (OrderByIDs drops absent ids)
	assert.Equal(t, p2.ID(), list[0].ID())
	assert.Equal(t, p1.ID(), list[1].ID())
}

func TestParameter_FindByProject_OrderedByIndex(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewParameter(pgxx.NewClient(pool))
	pid := id.NewProjectID()
	// Insert in reverse index order
	p2 := newParam(pid, 2)
	p0 := newParam(pid, 0)
	p1 := newParam(pid, 1)
	require.NoError(t, r.Save(ctx, p2))
	require.NoError(t, r.Save(ctx, p0))
	require.NoError(t, r.Save(ctx, p1))
	got, err := r.FindByProject(ctx, pid)
	require.NoError(t, err)
	require.NotNil(t, got)
	list := *got
	require.Len(t, list, 3)
	assert.Equal(t, 0, list[0].Index())
	assert.Equal(t, 1, list[1].Index())
	assert.Equal(t, 2, list[2].Index())
}

func TestParameter_FindByProject_Empty(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewParameter(pgxx.NewClient(pool))
	got, err := r.FindByProject(ctx, id.NewProjectID())
	require.NoError(t, err)
	assert.Nil(t, got)
}

func TestParameter_Remove(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewParameter(pgxx.NewClient(pool))
	pid := id.NewProjectID()
	p := newParam(pid, 0)
	require.NoError(t, r.Save(ctx, p))
	require.NoError(t, r.Remove(ctx, p.ID()))
	_, err := r.FindByID(ctx, p.ID())
	assert.ErrorIs(t, err, rerror.ErrNotFound)
}

func TestParameter_RemoveAll(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewParameter(pgxx.NewClient(pool))
	pid := id.NewProjectID()
	p1 := newParam(pid, 0)
	p2 := newParam(pid, 1)
	require.NoError(t, r.Save(ctx, p1))
	require.NoError(t, r.Save(ctx, p2))
	require.NoError(t, r.RemoveAll(ctx, id.ParameterIDList{p1.ID(), p2.ID()}))
	got, err := r.FindByProject(ctx, pid)
	require.NoError(t, err)
	assert.Nil(t, got)
}

func TestParameter_RemoveAllByProject(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewParameter(pgxx.NewClient(pool))
	pid1 := id.NewProjectID()
	pid2 := id.NewProjectID()
	p1 := newParam(pid1, 0)
	p2 := newParam(pid2, 0)
	require.NoError(t, r.Save(ctx, p1))
	require.NoError(t, r.Save(ctx, p2))
	require.NoError(t, r.RemoveAllByProject(ctx, pid1))
	got1, err := r.FindByProject(ctx, pid1)
	require.NoError(t, err)
	assert.Nil(t, got1)
	got2, err := r.FindByProject(ctx, pid2)
	require.NoError(t, err)
	assert.NotNil(t, got2)
}

func TestParameter_RemoveAll_Empty(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewParameter(pgxx.NewClient(pool))
	// Should not error on empty list
	require.NoError(t, r.RemoveAll(ctx, id.ParameterIDList{}))
}

func TestParameter_Save_WithDefaultValueAndConfig(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewParameter(pgxx.NewClient(pool))
	pid := id.NewProjectID()
	p, err := parameter.New().
		ID(id.NewParameterID()).
		ProjectID(pid).
		Name("with-meta").
		Type(parameter.TypeNumber).
		Index(0).
		DefaultValue(42.0).
		Config(map[string]interface{}{"min": 0.0, "max": 100.0}).
		CreatedAt(time.Now()).
		UpdatedAt(time.Now()).
		Build()
	require.NoError(t, err)
	require.NoError(t, r.Save(ctx, p))
	got, err := r.FindByID(ctx, p.ID())
	require.NoError(t, err)
	assert.NotNil(t, got.DefaultValue())
	assert.NotNil(t, got.Config())
}
