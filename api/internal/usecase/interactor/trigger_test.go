package interactor

import (
	"context"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo"
	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/mongox/mongotest"
	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/bson"
)

func TestTrigger_Create(t *testing.T) {
	ctx := context.Background()
	c := mongotest.Connect(t)(t)

	wid := accountdomain.NewWorkspaceID()
	did := id.NewDeploymentID()

	_, _ = c.Collection("deployment").InsertOne(ctx, bson.M{
		"id":          did.String(),
		"workspaceid": wid.String(),
		"workflowurl": "test.yaml",
		"version":     "v1",
		"createdat":   time.Now(),
	})

	repo := repo.Container{
		Trigger:    mongo.NewTrigger(mongox.NewClientWithDatabase(c)),
		Deployment: mongo.NewDeployment(mongox.NewClientWithDatabase(c)),
	}
	gateway := &gateway.Container{}
	job := NewJob(&repo, gateway)
	i := NewTrigger(&repo, gateway, job)

	param := interfaces.CreateTriggerParam{
		WorkspaceID:  wid,
		DeploymentID: did,
		Description:  "Daily trigger",
		EventSource:  "TIME_DRIVEN",
		TimeInterval: "EVERY_DAY",
	}

	got, err := i.Create(ctx, param, &usecase.Operator{})
	assert.NoError(t, err)
	assert.NotNil(t, got)
	assert.Equal(t, wid, got.Workspace())
	assert.Equal(t, did, got.Deployment())
	assert.Equal(t, "Daily trigger", got.Description())
	assert.Equal(t, trigger.EventSourceTypeTimeDriven, got.EventSource())
	assert.Equal(t, trigger.TimeIntervalEveryDay, *got.TimeInterval())

	param = interfaces.CreateTriggerParam{
		WorkspaceID:  wid,
		DeploymentID: did,
		Description:  "API trigger",
		EventSource:  "API_DRIVEN",
		AuthToken:    "token123",
	}

	got, err = i.Create(ctx, param, &usecase.Operator{})
	assert.NoError(t, err)
	assert.NotNil(t, got)
	assert.Equal(t, "API trigger", got.Description())
	assert.Equal(t, trigger.EventSourceTypeAPIDriven, got.EventSource())
	assert.Equal(t, "token123", *got.AuthToken())

	param.DeploymentID = id.NewDeploymentID()
	got, err = i.Create(ctx, param, &usecase.Operator{})
	assert.Error(t, err)
	assert.Nil(t, got)
}

func TestTrigger_Update(t *testing.T) {
	ctx := context.Background()
	c := mongotest.Connect(t)(t)

	tid := id.NewTriggerID()
	wid := accountdomain.NewWorkspaceID()
	did := id.NewDeploymentID()
	newDid := id.NewDeploymentID()

	_, _ = c.Collection("trigger").InsertOne(ctx, bson.M{
		"id":           tid.String(),
		"workspaceid":  wid.String(),
		"deploymentid": did.String(),
		"description":  "Original trigger",
		"eventsource":  "TIME_DRIVEN",
		"timeinterval": "EVERY_DAY",
		"createdat":    time.Now(),
	})

	_, _ = c.Collection("deployment").InsertMany(ctx, []any{
		bson.M{
			"id":          did.String(),
			"workspaceid": wid.String(),
			"workflowurl": "test.yaml",
			"version":     "v1",
			"createdat":   time.Now(),
		},
		bson.M{
			"id":          newDid.String(),
			"workspaceid": wid.String(),
			"workflowurl": "test2.yaml",
			"version":     "v1",
			"createdat":   time.Now(),
		},
	})

	repo := repo.Container{
		Trigger:    mongo.NewTrigger(mongox.NewClientWithDatabase(c)),
		Deployment: mongo.NewDeployment(mongox.NewClientWithDatabase(c)),
	}
	gateway := &gateway.Container{}
	job := NewJob(&repo, gateway)
	i := NewTrigger(&repo, gateway, job)

	// Test updating description and event source
	newDesc := "Updated trigger"
	param := interfaces.UpdateTriggerParam{
		ID:          tid,
		Description: &newDesc,
		EventSource: "API_DRIVEN",
		AuthToken:   "newtoken",
	}

	got, err := i.Update(ctx, param, &usecase.Operator{})
	assert.NoError(t, err)
	assert.Equal(t, "Updated trigger", got.Description())
	assert.Equal(t, trigger.EventSourceTypeAPIDriven, got.EventSource())
	assert.Equal(t, "newtoken", *got.AuthToken())
	assert.Nil(t, got.TimeInterval())

	// Test updating deployment
	param = interfaces.UpdateTriggerParam{
		ID:           tid,
		DeploymentID: &newDid,
		EventSource:  "TIME_DRIVEN",
		TimeInterval: "EVERY_HOUR",
	}

	got, err = i.Update(ctx, param, &usecase.Operator{})
	assert.NoError(t, err)
	assert.Equal(t, newDid, got.Deployment())
	assert.Equal(t, trigger.TimeIntervalEveryHour, *got.TimeInterval())

	// Test updating with invalid trigger ID
	param.ID = id.NewTriggerID()
	got, err = i.Update(ctx, param, &usecase.Operator{})
	assert.Error(t, err)
	assert.Nil(t, got)

	// Test updating with invalid deployment ID
	invalidDid := id.NewDeploymentID()
	param.ID = tid
	param.DeploymentID = &invalidDid
	got, err = i.Update(ctx, param, &usecase.Operator{})
	assert.Error(t, err)
	assert.Nil(t, got)
}

func TestTrigger_Fetch(t *testing.T) {
	ctx := context.Background()
	c := mongotest.Connect(t)(t)

	tid1 := id.NewTriggerID()
	tid2 := id.NewTriggerID()
	wid := accountdomain.NewWorkspaceID()
	did := id.NewDeploymentID()

	_, _ = c.Collection("trigger").InsertMany(ctx, []any{
		bson.M{
			"id":           tid1.String(),
			"workspaceid":  wid.String(),
			"deploymentid": did.String(),
			"description":  "Daily trigger",
			"eventsource":  "TIME_DRIVEN",
			"timeinterval": "EVERY_DAY",
			"createdat":    time.Now(),
		},
		bson.M{
			"id":           tid2.String(),
			"workspaceid":  wid.String(),
			"deploymentid": did.String(),
			"description":  "API trigger",
			"eventsource":  "API_DRIVEN",
			"authtoken":    "token123",
			"createdat":    time.Now(),
		},
	})

	repo := repo.Container{
		Trigger: mongo.NewTrigger(mongox.NewClientWithDatabase(c)),
	}
	gateway := &gateway.Container{}
	job := NewJob(&repo, gateway)
	i := NewTrigger(&repo, gateway, job)

	got, err := i.Fetch(ctx, []id.TriggerID{tid1, tid2}, &usecase.Operator{})
	assert.NoError(t, err)
	assert.Equal(t, 2, len(got))
	assert.Equal(t, tid1, got[0].ID())
	assert.Equal(t, "Daily trigger", got[0].Description())
	assert.Equal(t, tid2, got[1].ID())
	assert.Equal(t, "API trigger", got[1].Description())
}

func TestTrigger_Delete(t *testing.T) {
	ctx := context.Background()
	c := mongotest.Connect(t)(t)

	tid := id.NewTriggerID()
	wid := accountdomain.NewWorkspaceID()
	did := id.NewDeploymentID()

	_, _ = c.Collection("trigger").InsertOne(ctx, bson.M{
		"id":           tid.String(),
		"workspaceid":  wid.String(),
		"deploymentid": did.String(),
		"eventsource":  "TIME_DRIVEN",
		"timeinterval": "EVERY_DAY",
		"createdat":    time.Now(),
	})

	repo := repo.Container{
		Trigger: mongo.NewTrigger(mongox.NewClientWithDatabase(c)),
	}
	gateway := &gateway.Container{}
	job := NewJob(&repo, gateway)
	i := NewTrigger(&repo, gateway, job)

	err := i.Delete(ctx, tid, &usecase.Operator{})
	assert.NoError(t, err)

	var count int64
	count, err = c.Collection("trigger").CountDocuments(ctx, bson.M{"id": tid.String()})
	assert.NoError(t, err)
	assert.Equal(t, int64(0), count)

	err = i.Delete(ctx, id.NewTriggerID(), &usecase.Operator{})
	assert.NoError(t, err)
}
