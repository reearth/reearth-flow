package postgres_test

import (
	"context"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newWf(wfid id.WorkflowID, pid id.ProjectID, wsid accountsid.WorkspaceID) *workflow.Workflow {
	return workflow.NewWorkflow(wfid, pid, wsid, "gs://bucket/workflow.yaml")
}

func TestWorkflow_Save_FindByID(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	wfid := id.NewWorkflowID()
	pid := id.NewProjectID()
	wsid := accountsid.NewWorkspaceID()
	wf := newWf(wfid, pid, wsid)
	r := postgres.NewWorkflow(pgxx.NewClient(pool))

	require.NoError(t, r.Save(ctx, wf))

	got, err := r.FindByID(ctx, wfid)
	require.NoError(t, err)
	require.NotNil(t, got)
	assert.Equal(t, wfid, got.ID())
	assert.Equal(t, pid, got.Project())
	assert.Equal(t, wsid, got.Workspace())
	assert.Equal(t, "gs://bucket/workflow.yaml", got.URL())
}

func TestWorkflow_FindByID_NotFound(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewWorkflow(pgxx.NewClient(pool))

	got, err := r.FindByID(ctx, id.NewWorkflowID())
	assert.Nil(t, got)
	assert.ErrorIs(t, err, rerror.ErrNotFound)
}

func TestWorkflow_Save_Upsert(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	wfid := id.NewWorkflowID()
	pid := id.NewProjectID()
	wsid := accountsid.NewWorkspaceID()
	wf := newWf(wfid, pid, wsid)
	r := postgres.NewWorkflow(pgxx.NewClient(pool))

	require.NoError(t, r.Save(ctx, wf))

	// Update URL via a new workflow value with same ID.
	wf2 := workflow.NewWorkflow(wfid, pid, wsid, "gs://bucket/workflow_v2.yaml")
	require.NoError(t, r.Save(ctx, wf2))

	got, err := r.FindByID(ctx, wfid)
	require.NoError(t, err)
	assert.Equal(t, "gs://bucket/workflow_v2.yaml", got.URL())
}

func TestWorkflow_Remove(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	wfid := id.NewWorkflowID()
	pid := id.NewProjectID()
	wsid := accountsid.NewWorkspaceID()
	wf := newWf(wfid, pid, wsid)
	r := postgres.NewWorkflow(pgxx.NewClient(pool))

	require.NoError(t, r.Save(ctx, wf))
	require.NoError(t, r.Remove(ctx, wfid))

	got, err := r.FindByID(ctx, wfid)
	assert.Nil(t, got)
	assert.ErrorIs(t, err, rerror.ErrNotFound)
}

func TestWorkflow_Filtered_CanRead(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	wfid := id.NewWorkflowID()
	pid := id.NewProjectID()
	wsid := accountsid.NewWorkspaceID()
	wf := newWf(wfid, pid, wsid)
	r := postgres.NewWorkflow(pgxx.NewClient(pool))

	require.NoError(t, r.Save(ctx, wf))

	// Filter to a different workspace — FindByID should return ErrNotFound.
	otherWS := accountsid.NewWorkspaceID()
	filtered := r.Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{otherWS},
		Writable: accountsid.WorkspaceIDList{otherWS},
	})
	got, err := filtered.FindByID(ctx, wfid)
	assert.Nil(t, got)
	assert.ErrorIs(t, err, rerror.ErrNotFound)

	// Filter to correct workspace — FindByID should succeed.
	filtered2 := r.Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{wsid},
		Writable: accountsid.WorkspaceIDList{wsid},
	})
	got2, err := filtered2.FindByID(ctx, wfid)
	require.NoError(t, err)
	assert.Equal(t, wfid, got2.ID())
}

func TestWorkflow_Filtered_CanWrite(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	wfid := id.NewWorkflowID()
	pid := id.NewProjectID()
	wsid := accountsid.NewWorkspaceID()
	wf := newWf(wfid, pid, wsid)

	// Save without filter succeeds.
	r := postgres.NewWorkflow(pgxx.NewClient(pool))
	require.NoError(t, r.Save(ctx, wf))

	// Save via filter with different writable workspace — must be denied.
	otherWS := accountsid.NewWorkspaceID()
	filtered := r.Filtered(repo.WorkspaceFilter{
		Writable: accountsid.WorkspaceIDList{otherWS},
	})
	err := filtered.Save(ctx, wf)
	assert.ErrorIs(t, err, repo.ErrOperationDenied)
}
