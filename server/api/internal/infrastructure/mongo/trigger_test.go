package mongo

import (
	"context"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/mongox/mongotest"
	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/bson"
)

func TestTrigger_FindByID(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()

	tid := id.NewTriggerID()
	wid := accountsid.NewWorkspaceID()

	_, _ = c.Collection("trigger").InsertOne(ctx, bson.M{
		"id":          tid.String(),
		"workspaceid": wid.String(),
		"createdat":   time.Now(),
	})

	r := NewTrigger(mongox.NewClientWithDatabase(c))

	got, err := r.FindByID(ctx, tid)
	assert.NoError(t, err)
	assert.Equal(t, tid, got.ID())
}

func TestTrigger_FindByIDs(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()

	tid1 := id.NewTriggerID()
	tid2 := id.NewTriggerID()
	wid := accountsid.NewWorkspaceID()

	_, _ = c.Collection("trigger").InsertMany(ctx, []any{
		bson.M{"id": tid1.String(), "workspaceid": wid.String()},
		bson.M{"id": tid2.String(), "workspaceid": wid.String()},
	})

	r := NewTrigger(mongox.NewClientWithDatabase(c))

	got, err := r.FindByIDs(ctx, id.TriggerIDList{tid1, tid2})
	assert.NoError(t, err)
	assert.Equal(t, 2, len(got))
	assert.Equal(t, tid1, got[0].ID())
	assert.Equal(t, tid2, got[1].ID())
}

func TestTrigger_FindByWorkspace(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()

	wid := accountsid.NewWorkspaceID()
	wid2 := accountsid.NewWorkspaceID()

	_, _ = c.Collection("trigger").InsertMany(ctx, []any{
		bson.M{"id": "t1", "workspaceid": wid.String(), "eventsource": "TIME_DRIVEN"},
		bson.M{"id": "t2", "workspaceid": wid.String(), "eventsource": "API_DRIVEN"},
		bson.M{"id": "t3", "workspaceid": wid2.String(), "eventsource": "TIME_DRIVEN"},
	})

	r := NewTrigger(mongox.NewClientWithDatabase(c))

	got, pageInfo, err := r.FindByWorkspace(ctx, wid, nil, nil)
	assert.NoError(t, err)
	assert.NotNil(t, pageInfo)
	assert.Equal(t, 2, len(got))
	assert.Equal(t, "t1", got[0].ID().String())
	assert.Equal(t, "t2", got[1].ID().String())
}

func TestTrigger_Save(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()

	tid := id.NewTriggerID()
	wid := accountsid.NewWorkspaceID()

	tr := trigger.New().
		ID(tid).
		Workspace(wid).
		EventSource(trigger.EventSourceTypeTimeDriven).
		UpdatedAt(time.Now()).
		MustBuild()

	r := NewTrigger(mongox.NewClientWithDatabase(c))

	err := r.Save(ctx, tr)
	assert.NoError(t, err)

	got, err := r.FindByID(ctx, tid)
	assert.NoError(t, err)
	assert.Equal(t, tr.ID(), got.ID())
	assert.Equal(t, tr.Workspace(), got.Workspace())
}

func TestTrigger_Remove(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()

	tid := id.NewTriggerID()

	_, _ = c.Collection("trigger").InsertOne(ctx, bson.M{"id": tid.String()})

	r := NewTrigger(mongox.NewClientWithDatabase(c))

	err := r.Remove(ctx, tid)
	assert.NoError(t, err)

	got, err := r.FindByID(ctx, tid)
	assert.Error(t, err)
	assert.Nil(t, got)
}

func TestTrigger_Remove_WithWorkspaceFilter(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()

	wid1 := accountsid.NewWorkspaceID()
	wid2 := accountsid.NewWorkspaceID()
	tid1 := id.NewTriggerID()
	tid2 := id.NewTriggerID()

	_, _ = c.Collection("trigger").InsertMany(ctx, []any{
		bson.M{"id": tid1.String(), "workspaceid": wid1.String()},
		bson.M{"id": tid2.String(), "workspaceid": wid2.String()},
	})

	filter := repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{wid1},
		Writable: accountsid.WorkspaceIDList{wid1},
	}
	r := NewTrigger(mongox.NewClientWithDatabase(c)).Filtered(filter)

	err := r.Remove(ctx, tid1)
	assert.NoError(t, err)

	got, err := r.FindByID(ctx, tid1)
	assert.Error(t, err)
	assert.Nil(t, got)

	err = r.Remove(ctx, tid2)
	assert.NoError(t, err)

	base := NewTrigger(mongox.NewClientWithDatabase(c))
	got, err = base.FindByID(ctx, tid2)
	assert.NoError(t, err)
	assert.NotNil(t, got)
}

func TestTrigger_DeploymentUpdates(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()

	wid := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	tid := id.NewTriggerID()

	initialDeployment := deployment.New().
		ID(did).
		Workspace(wid).
		Version("v1").
		WorkflowURL("initial-workflow.yaml").
		UpdatedAt(time.Now()).
		MustBuild()

	tr := trigger.New().
		ID(tid).
		Workspace(wid).
		Deployment(did).
		EventSource(trigger.EventSourceTypeTimeDriven).
		TimeInterval(trigger.TimeIntervalEveryDay).
		UpdatedAt(time.Now()).
		MustBuild()

	deploymentRepo := NewDeployment(mongox.NewClientWithDatabase(c))
	triggerRepo := NewTrigger(mongox.NewClientWithDatabase(c))

	err := deploymentRepo.Save(ctx, initialDeployment)
	assert.NoError(t, err)
	err = triggerRepo.Save(ctx, tr)
	assert.NoError(t, err)

	updatedDeployment := deployment.New().
		ID(did).
		Workspace(wid).
		Version("v2").
		WorkflowURL("updated-workflow.yaml").
		UpdatedAt(time.Now()).
		MustBuild()

	err = deploymentRepo.Save(ctx, updatedDeployment)
	assert.NoError(t, err)

	gotTrigger, err := triggerRepo.FindByID(ctx, tid)
	assert.NoError(t, err)
	assert.Equal(t, did, gotTrigger.Deployment())

	gotDeployment, err := deploymentRepo.FindByID(ctx, gotTrigger.Deployment())
	assert.NoError(t, err)
	assert.Equal(t, "v2", gotDeployment.Version())
	assert.Equal(t, "updated-workflow.yaml", gotDeployment.WorkflowURL())
}
