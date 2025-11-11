package interactor

import (
	"context"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearth-flow/api/pkg/user"
	"github.com/reearth/reearthx/appx"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/mongox/mongotest"
	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/bson"
)

func TestTrigger_Create(t *testing.T) {
	mockAuthInfo := &appx.AuthInfo{
		Token: "token",
	}
	mockUser := user.New().NewID().Name("hoge").Email("abc@bb.cc").MustBuild()

	ctx := context.Background()
	ctx = adapter.AttachAuthInfo(ctx, mockAuthInfo)
	ctx = adapter.AttachUser(ctx, mockUser)

	c := mongotest.Connect(t)(t)

	wid := id.NewWorkspaceID()
	did := id.NewDeploymentID()

	testVars := map[string]string{
		"VAR_1": "test_val_1",
		"VAR_2": "test_val_2",
	}

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
	mockPermissionCheckerTrue := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
		return true, nil
	})
	job := NewJob(&repo, gateway, mockPermissionCheckerTrue)
	i := NewTrigger(&repo, gateway, job, mockPermissionCheckerTrue)

	param := interfaces.CreateTriggerParam{
		WorkspaceID:  wid,
		DeploymentID: did,
		Description:  "Daily trigger",
		EventSource:  "TIME_DRIVEN",
		TimeInterval: "EVERY_DAY",
		Variables:    testVars,
	}

	got, err := i.Create(ctx, param)
	assert.NoError(t, err)
	assert.NotNil(t, got)
	assert.Equal(t, wid, got.Workspace())
	assert.Equal(t, did, got.Deployment())
	assert.Equal(t, "Daily trigger", got.Description())
	assert.Equal(t, trigger.EventSourceTypeTimeDriven, got.EventSource())
	assert.Equal(t, trigger.TimeIntervalEveryDay, *got.TimeInterval())
	assert.Equal(t, testVars, got.Variables())

	param = interfaces.CreateTriggerParam{
		WorkspaceID:  wid,
		DeploymentID: did,
		Description:  "API trigger",
		EventSource:  "API_DRIVEN",
		AuthToken:    "token123",
		Variables:    testVars,
	}

	got, err = i.Create(ctx, param)
	assert.NoError(t, err)
	assert.NotNil(t, got)
	assert.Equal(t, "API trigger", got.Description())
	assert.Equal(t, trigger.EventSourceTypeAPIDriven, got.EventSource())
	assert.Equal(t, "token123", *got.AuthToken())
	assert.Equal(t, testVars, got.Variables())

	param.DeploymentID = id.NewDeploymentID()
	got, err = i.Create(ctx, param)
	assert.Error(t, err)
	assert.Nil(t, got)
}

func TestTrigger_Update(t *testing.T) {
	mockAuthInfo := &appx.AuthInfo{
		Token: "token",
	}
	mockUser := user.New().NewID().Name("hoge").Email("abc@bb.cc").MustBuild()

	ctx := context.Background()
	ctx = adapter.AttachAuthInfo(ctx, mockAuthInfo)
	ctx = adapter.AttachUser(ctx, mockUser)

	c := mongotest.Connect(t)(t)

	tid := id.NewTriggerID()
	wid := id.NewWorkspaceID()
	did := id.NewDeploymentID()
	newDid := id.NewDeploymentID()
	initialVars := map[string]string{"INIT_KEY": "initial_value"}

	_, _ = c.Collection("trigger").InsertOne(ctx, bson.M{
		"id":           tid.String(),
		"workspaceid":  wid.String(),
		"deploymentid": did.String(),
		"description":  "Original trigger",
		"eventsource":  "TIME_DRIVEN",
		"timeinterval": "EVERY_DAY",
		"variables":    initialVars,
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
	mockPermissionCheckerTrue := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
		return true, nil
	})
	job := NewJob(&repo, gateway, mockPermissionCheckerTrue)
	i := NewTrigger(&repo, gateway, job, mockPermissionCheckerTrue)

	// Test updating description and event source
	newDesc := "Updated trigger"
	updateVars := map[string]string{"NEW_VAR": "updated"}
	param := interfaces.UpdateTriggerParam{
		ID:          tid,
		Description: &newDesc,
		EventSource: "API_DRIVEN",
		AuthToken:   "newtoken",
		Variables:   updateVars,
	}

	got, err := i.Update(ctx, param)
	assert.NoError(t, err)
	assert.Equal(t, "Updated trigger", got.Description())
	assert.Equal(t, trigger.EventSourceTypeAPIDriven, got.EventSource())
	assert.Equal(t, "newtoken", *got.AuthToken())
	assert.Nil(t, got.TimeInterval())
	assert.Equal(t, updateVars, got.Variables())

	// Test updating deployment
	param = interfaces.UpdateTriggerParam{
		ID:           tid,
		DeploymentID: &newDid,
		EventSource:  "TIME_DRIVEN",
		TimeInterval: "EVERY_HOUR",
		Variables:    nil,
	}

	got, err = i.Update(ctx, param)
	assert.NoError(t, err)
	assert.Equal(t, newDid, got.Deployment())
	assert.Equal(t, trigger.TimeIntervalEveryHour, *got.TimeInterval())
	assert.Equal(t, updateVars, got.Variables())

	// Test updating with invalid trigger ID
	param.ID = id.NewTriggerID()
	param.Variables = updateVars
	got, err = i.Update(ctx, param)
	assert.Error(t, err)
	assert.Nil(t, got)

	// Test updating with invalid deployment ID
	invalidDid := id.NewDeploymentID()
	param.ID = tid
	param.DeploymentID = &invalidDid
	got, err = i.Update(ctx, param)
	assert.Error(t, err)
	assert.Nil(t, got)
}

func TestTrigger_Fetch(t *testing.T) {
	mockAuthInfo := &appx.AuthInfo{
		Token: "token",
	}
	mockUser := user.New().NewID().Name("hoge").Email("abc@bb.cc").MustBuild()

	ctx := context.Background()
	ctx = adapter.AttachAuthInfo(ctx, mockAuthInfo)
	ctx = adapter.AttachUser(ctx, mockUser)

	c := mongotest.Connect(t)(t)

	tid1 := id.NewTriggerID()
	tid2 := id.NewTriggerID()
	wid := id.NewWorkspaceID()
	did := id.NewDeploymentID()
	testVars := map[string]string{"FETCH_VAR": "fetched_value"}

	_, _ = c.Collection("trigger").InsertMany(ctx, []any{
		bson.M{
			"id":           tid1.String(),
			"workspaceid":  wid.String(),
			"deploymentid": did.String(),
			"description":  "Daily trigger",
			"eventsource":  "TIME_DRIVEN",
			"timeinterval": "EVERY_DAY",
			"createdat":    time.Now(),
			"variables":    testVars,
		},
		bson.M{
			"id":           tid2.String(),
			"workspaceid":  wid.String(),
			"deploymentid": did.String(),
			"description":  "API trigger",
			"eventsource":  "API_DRIVEN",
			"authtoken":    "token123",
			"createdat":    time.Now(),
			"variables":    testVars,
		},
	})

	repo := repo.Container{
		Trigger: mongo.NewTrigger(mongox.NewClientWithDatabase(c)),
	}
	gateway := &gateway.Container{}
	mockPermissionCheckerTrue := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
		return true, nil
	})
	job := NewJob(&repo, gateway, mockPermissionCheckerTrue)
	i := NewTrigger(&repo, gateway, job, mockPermissionCheckerTrue)

	got, err := i.Fetch(ctx, []id.TriggerID{tid1, tid2})
	assert.NoError(t, err)
	assert.Equal(t, 2, len(got))
	assert.Equal(t, tid1, got[0].ID())
	assert.Equal(t, "Daily trigger", got[0].Description())
	assert.Equal(t, testVars, got[0].Variables())
	assert.Equal(t, tid2, got[1].ID())
	assert.Equal(t, "API trigger", got[1].Description())
	assert.Equal(t, testVars, got[1].Variables())
}

func TestTrigger_Delete(t *testing.T) {
	mockAuthInfo := &appx.AuthInfo{
		Token: "token",
	}
	mockUser := user.New().NewID().Name("hoge").Email("abc@bb.cc").MustBuild()

	ctx := context.Background()
	ctx = adapter.AttachAuthInfo(ctx, mockAuthInfo)
	ctx = adapter.AttachUser(ctx, mockUser)

	c := mongotest.Connect(t)(t)

	tid := id.NewTriggerID()
	wid := id.NewWorkspaceID()
	did := id.NewDeploymentID()

	_, _ = c.Collection("trigger").InsertOne(ctx, bson.M{
		"id":           tid.String(),
		"workspaceid":  wid.String(),
		"deploymentid": did.String(),
		"eventsource":  "TIME_DRIVEN",
		"timeinterval": "EVERY_DAY",
		"createdat":    time.Now(),
		"variables":    map[string]string{"del_var": "test"},
	})

	repo := repo.Container{
		Trigger: mongo.NewTrigger(mongox.NewClientWithDatabase(c)),
	}
	gateway := &gateway.Container{}
	mockPermissionCheckerTrue := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
		return true, nil
	})
	job := NewJob(&repo, gateway, mockPermissionCheckerTrue)
	i := NewTrigger(&repo, gateway, job, mockPermissionCheckerTrue)

	err := i.Delete(ctx, tid)
	assert.NoError(t, err)

	var count int64
	count, err = c.Collection("trigger").CountDocuments(ctx, bson.M{"id": tid.String()})
	assert.NoError(t, err)
	assert.Equal(t, int64(0), count)

	err = i.Delete(ctx, id.NewTriggerID())
	assert.NoError(t, err)
}
