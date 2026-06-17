package postgres_test

import (
	"context"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
	"github.com/reearth/reearthx/pgxx"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newWorkerCfg(wid id.WorkerConfigID) *workerconfig.WorkerConfig {
	machineType := "n2-standard-4"
	cpu := 1000
	mem := 2048
	cfg, err := workerconfig.NewBuilder().
		ID(wid).
		MachineType(&machineType).
		ComputeCpuMilli(&cpu).
		ComputeMemoryMib(&mem).
		CreatedAt(time.Now()).
		UpdatedAt(time.Now()).
		Build()
	if err != nil {
		panic(err)
	}
	return cfg
}

func TestWorkerConfig_Save_FindByID(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	wid := id.NewWorkerConfigID()
	cfg := newWorkerCfg(wid)
	r := postgres.NewWorkerConfig(pgxx.NewClient(pool))
	require.NoError(t, r.Save(ctx, cfg))
	got, err := r.FindByID(ctx, wid)
	require.NoError(t, err)
	require.NotNil(t, got)
	assert.Equal(t, wid, got.ID())
	assert.Equal(t, "n2-standard-4", *got.MachineType())
	assert.Equal(t, 1000, *got.ComputeCpuMilli())
	assert.Equal(t, 2048, *got.ComputeMemoryMib())
}

func TestWorkerConfig_FindByID_NotFound(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	got, err := postgres.NewWorkerConfig(pgxx.NewClient(pool)).FindByID(context.Background(), id.NewWorkerConfigID())
	assert.Nil(t, got)
	assert.Error(t, err)
}

func TestWorkerConfig_FindByIDs_Order(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewWorkerConfig(pgxx.NewClient(pool))
	wid1 := id.NewWorkerConfigID()
	wid2 := id.NewWorkerConfigID()
	require.NoError(t, r.Save(ctx, newWorkerCfg(wid1)))
	require.NoError(t, r.Save(ctx, newWorkerCfg(wid2)))
	missing := id.NewWorkerConfigID()
	got, err := r.FindByIDs(ctx, []id.WorkerConfigID{wid2, missing, wid1})
	require.NoError(t, err)
	require.Len(t, got, 3)
	assert.Equal(t, wid2, got[0].ID())
	assert.Nil(t, got[1])
	assert.Equal(t, wid1, got[2].ID())
}

func TestWorkerConfig_FindAll(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewWorkerConfig(pgxx.NewClient(pool))

	// Empty table returns nil, no error.
	got, err := r.FindAll(ctx)
	require.NoError(t, err)
	assert.Nil(t, got)

	wid := id.NewWorkerConfigID()
	require.NoError(t, r.Save(ctx, newWorkerCfg(wid)))
	got, err = r.FindAll(ctx)
	require.NoError(t, err)
	require.NotNil(t, got)
}

func TestWorkerConfig_Save_Upsert(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewWorkerConfig(pgxx.NewClient(pool))
	wid := id.NewWorkerConfigID()
	cfg := newWorkerCfg(wid)
	require.NoError(t, r.Save(ctx, cfg))

	// Update: change machine type.
	newType := "c2-standard-8"
	cfg.SetMachineType(&newType)
	require.NoError(t, r.Save(ctx, cfg))

	got, err := r.FindByID(ctx, wid)
	require.NoError(t, err)
	assert.Equal(t, "c2-standard-8", *got.MachineType())
}

func TestWorkerConfig_Remove(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewWorkerConfig(pgxx.NewClient(pool))
	wid := id.NewWorkerConfigID()
	require.NoError(t, r.Save(ctx, newWorkerCfg(wid)))
	require.NoError(t, r.Remove(ctx, wid))
	got, err := r.FindByID(ctx, wid)
	assert.Error(t, err)
	assert.Nil(t, got)
}

func TestWorkerConfig_Save_NilFields(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewWorkerConfig(pgxx.NewClient(pool))
	wid := id.NewWorkerConfigID()
	cfg, err := workerconfig.NewBuilder().
		ID(wid).
		CreatedAt(time.Now()).
		UpdatedAt(time.Now()).
		Build()
	require.NoError(t, err)
	require.NoError(t, r.Save(ctx, cfg))
	got, err := r.FindByID(ctx, wid)
	require.NoError(t, err)
	assert.Nil(t, got.MachineType())
	assert.Nil(t, got.ComputeCpuMilli())
}
