package memory

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
)

func TestWorkflow_SaveAndFindByID(t *testing.T) {
	ctx := context.Background()
	wsID, _ := accountdomain.WorkspaceIDFrom("workspace1")
	wfID := id.NewWorkflowID()
	wf := workflow.NewWorkflow(wfID, id.NewProjectID(), wsID, "workflow_url", id.NewGraphID())

	repoWF := NewWorkflow()

	t.Run("Save and FindByID", func(t *testing.T) {
		err := repoWF.Save(ctx, wf)
		assert.NoError(t, err)

		result, err := repoWF.FindByID(ctx, wfID)
		assert.NoError(t, err)
		assert.Equal(t, wf, result)
	})
}

func TestWorkflow_Remove(t *testing.T) {
	ctx := context.Background()
	wsID, _ := accountdomain.WorkspaceIDFrom("workspace1")
	wfID := id.NewWorkflowID()
	wf := workflow.NewWorkflow(wfID, id.NewProjectID(), wsID, "workflow_url", id.NewGraphID())
	repoWF := NewWorkflow()

	err := repoWF.Save(ctx, wf)
	assert.NoError(t, err)

	t.Run("Remove existing workflow", func(t *testing.T) {
		err := repoWF.Remove(ctx, wfID)
		assert.NoError(t, err)

		_, err = repoWF.FindByID(ctx, wfID)
		assert.Error(t, err)
		assert.Equal(t, rerror.ErrNotFound.Error(), err.Error())
	})
}

func TestWorkflow_Filtered(t *testing.T) {
	ctx := context.Background()
	wsID, _ := accountdomain.WorkspaceIDFrom("workspace1")
	wfID := id.NewWorkflowID()
	wf := workflow.NewWorkflow(wfID, id.NewProjectID(), wsID, "workflow_url", id.NewGraphID())
	repoWF := NewWorkflow()

	err := repoWF.Save(ctx, wf)
	assert.NoError(t, err)

	t.Run("Filtered denies read", func(t *testing.T) {
		filteredRepo := repoWF.Filtered(repo.WorkspaceFilter{Readable: []accountdomain.WorkspaceID{}})
		result, err := filteredRepo.FindByID(ctx, wfID)
		assert.Error(t, err)
		assert.Nil(t, result)
	})

	t.Run("Filtered allows read", func(t *testing.T) {
		filteredRepo := repoWF.Filtered(repo.WorkspaceFilter{Readable: []accountdomain.WorkspaceID{wsID}})
		result, err := filteredRepo.FindByID(ctx, wfID)
		assert.NoError(t, err)
		assert.Equal(t, wf, result)
	})
}

func TestWorkflow_OperationDenied(t *testing.T) {
	ctx := context.Background()
	wsID, _ := accountdomain.WorkspaceIDFrom("workspace1")
	wfID := id.NewWorkflowID()
	wf := workflow.NewWorkflow(wfID, id.NewProjectID(), wsID, "workflow_url", id.NewGraphID())

	repoWF := NewWorkflow()
	repoWF.f = repo.WorkspaceFilter{Writable: []accountdomain.WorkspaceID{}}

	t.Run("Save operation denied", func(t *testing.T) {
		err := repoWF.Save(ctx, wf)
		assert.Error(t, err)
		assert.Equal(t, repo.ErrOperationDenied, err)
	})

	repoWFAllow := NewWorkflow()
	repoWFAllow.f = repo.WorkspaceFilter{Writable: []accountdomain.WorkspaceID{wsID}}
	err := repoWFAllow.Save(ctx, wf)
	assert.NoError(t, err)

	t.Run("Remove operation denied", func(t *testing.T) {
		filteredRepo := NewWorkflow()
		filteredRepo.data = repoWFAllow.data
		filteredRepo.f = repo.WorkspaceFilter{Writable: []accountdomain.WorkspaceID{}}
		err := filteredRepo.Remove(ctx, wfID)
		assert.Error(t, err)
		assert.Equal(t, repo.ErrOperationDenied, err)
	})
}
