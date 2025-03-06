package mongo

import (
	"context"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/mongox/mongotest"
	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/bson"
)

func TestWorkflow_FindByID(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()

	wfID := id.NewWorkflowID()
	wsID := accountdomain.NewWorkspaceID()

	_, err := c.Collection("workflow").InsertOne(ctx, bson.M{
		"id":        wfID.String(),
		"workspace": wsID.String(),
		"createdat": time.Now(),
	})
	assert.NoError(t, err)

	repo := NewWorkflow(mongox.NewClientWithDatabase(c))
	result, err := repo.FindByID(ctx, wfID)
	assert.NoError(t, err)
	assert.Equal(t, wfID, result.ID)
}

func TestWorkflow_Save(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()

	wfID := id.NewWorkflowID()
	wsID := accountdomain.NewWorkspaceID()

	wf := workflow.NewWorkflow(wfID, id.NewProjectID(), wsID, "workflow_url", id.NewGraphID())

	repo := NewWorkflow(mongox.NewClientWithDatabase(c))
	err := repo.Save(ctx, wf)
	assert.NoError(t, err)

	result, err := repo.FindByID(ctx, wfID)
	assert.NoError(t, err)
	assert.Equal(t, wf.ID(), result.ID())
	assert.Equal(t, wf.Workspace(), result.Workspace())
}

func TestWorkflow_Remove(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()

	wfID := id.NewWorkflowID()
	wsID := accountdomain.NewWorkspaceID()

	_, err := c.Collection("workflow").InsertOne(ctx, bson.M{
		"id":        wfID.String(),
		"workspace": wsID.String(),
	})
	assert.NoError(t, err)

	repo := NewWorkflow(mongox.NewClientWithDatabase(c))
	err = repo.Remove(ctx, wfID)
	assert.NoError(t, err)

	result, err := repo.FindByID(ctx, wfID)
	assert.Error(t, err)
	assert.Nil(t, result)
}

func TestWorkflow_Init(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()

	repo := NewWorkflow(mongox.NewClientWithDatabase(c))
	err := repo.Init(ctx)
	assert.NoError(t, err)
}
