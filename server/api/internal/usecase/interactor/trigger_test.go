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
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearth-flow/api/pkg/user"
	"github.com/reearth/reearth-flow/api/pkg/variable"
	"github.com/reearth/reearthx/appx"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/mongox/mongotest"
	"github.com/samber/lo"
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

	testVars := []variable.Variable{
		{Key: "VAR_1", Type: parameter.TypeText, Value: "test_val_1"},
		{Key: "VAR_2", Type: parameter.TypeText, Value: "test_val_2"},
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
		Enabled:      true,
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
	assert.True(t, got.Enabled())
	assert.Equal(t, variablesToSimpleMap(testVars), variablesToSimpleMap(got.Variables()))
	assert.False(t, got.CreatedAt().IsZero())
	assert.False(t, got.UpdatedAt().IsZero())

	param = interfaces.CreateTriggerParam{
		WorkspaceID:  wid,
		DeploymentID: did,
		Description:  "API trigger",
		EventSource:  "API_DRIVEN",
		AuthToken:    "token123",
		Enabled:      false,
		Variables:    testVars,
	}

	got, err = i.Create(ctx, param)
	assert.NoError(t, err)
	assert.NotNil(t, got)
	assert.Equal(t, "API trigger", got.Description())
	assert.Equal(t, trigger.EventSourceTypeAPIDriven, got.EventSource())
	assert.Equal(t, "token123", *got.AuthToken())
	assert.Equal(t, variablesToSimpleMap(testVars), variablesToSimpleMap(got.Variables()))
	assert.False(t, got.Enabled())

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

	initialVarsDoc := []bson.M{
		{
			"key":   "INIT_KEY",
			"type":  string(parameter.TypeText),
			"value": "initial_value",
		},
	}

	createdTime := time.Now()
	_, _ = c.Collection("trigger").InsertOne(ctx, bson.M{
		"id":           tid.String(),
		"workspaceid":  wid.String(),
		"deploymentid": did.String(),
		"description":  "Original trigger",
		"eventsource":  "TIME_DRIVEN",
		"timeinterval": "EVERY_DAY",
		"enabled":      true,
		"variables":    initialVarsDoc,
		"createdat":    createdTime,
		"updatedat":    createdTime,
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
	updateVars := []variable.Variable{
		{Key: "NEW_VAR", Type: parameter.TypeText, Value: "updated"},
	}
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
	assert.True(t, got.Enabled())
	assert.Equal(t, variablesToSimpleMap(updateVars), variablesToSimpleMap(got.Variables()))
	assert.Equal(t, createdTime, got.CreatedAt())
	assert.True(t, got.UpdatedAt().After(createdTime))

	// Test updating deployment
	param = interfaces.UpdateTriggerParam{
		ID:           tid,
		DeploymentID: &newDid,
		EventSource:  "TIME_DRIVEN",
		TimeInterval: "EVERY_HOUR",
		Enabled:      lo.ToPtr(false),
		Variables:    nil,
	}

	got, err = i.Update(ctx, param)
	assert.NoError(t, err)
	assert.Equal(t, newDid, got.Deployment())
	assert.Equal(t, trigger.TimeIntervalEveryHour, *got.TimeInterval())
	assert.False(t, got.Enabled())
	assert.Equal(t, variablesToSimpleMap(updateVars), variablesToSimpleMap(got.Variables()))
	assert.Equal(t, createdTime, got.CreatedAt())
	assert.True(t, got.UpdatedAt().After(createdTime))

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
	testVarsDoc := []bson.M{
		{
			"key":   "FETCH_VAR",
			"type":  string(parameter.TypeText),
			"value": "fetched_value",
		},
	}
	createdTime := time.Now()

	_, _ = c.Collection("trigger").InsertMany(ctx, []any{
		bson.M{
			"id":           tid1.String(),
			"workspaceid":  wid.String(),
			"deploymentid": did.String(),
			"description":  "Daily trigger",
			"eventsource":  "TIME_DRIVEN",
			"timeinterval": "EVERY_DAY",
			"createdat":    createdTime,
			"updatedat":    createdTime,
			"enabled":      true,
			"variables":    testVarsDoc,
		},
		bson.M{
			"id":           tid2.String(),
			"workspaceid":  wid.String(),
			"deploymentid": did.String(),
			"description":  "API trigger",
			"eventsource":  "API_DRIVEN",
			"authtoken":    "token123",
			"createdat":    createdTime,
			"updatedat":    createdTime,
			"enabled":      false,
			"variables":    testVarsDoc,
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
	assert.True(t, got[0].Enabled())
	vars0 := variablesToSimpleMap(got[0].Variables())
	assert.Equal(t, map[string]interface{}{"FETCH_VAR": "fetched_value"}, vars0)
	assert.Equal(t, createdTime, got[0].CreatedAt())
	assert.Equal(t, createdTime, got[0].UpdatedAt())

	assert.Equal(t, tid2, got[1].ID())
	assert.Equal(t, "API trigger", got[1].Description())
	assert.False(t, got[1].Enabled())
	vars1 := variablesToSimpleMap(got[1].Variables())
	assert.Equal(t, map[string]interface{}{"FETCH_VAR": "fetched_value"}, vars1)
	assert.Equal(t, createdTime, got[1].CreatedAt())
	assert.Equal(t, createdTime, got[1].UpdatedAt())
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

	initialVarsDoc := []bson.M{
		{
			"key":   "INIT_KEY",
			"type":  string(parameter.TypeText),
			"value": "initial_value",
		},
	}

	_, _ = c.Collection("trigger").InsertOne(ctx, bson.M{
		"id":           tid.String(),
		"workspaceid":  wid.String(),
		"deploymentid": did.String(),
		"eventsource":  "TIME_DRIVEN",
		"timeinterval": "EVERY_DAY",
		"createdat":    time.Now(),
		"variables":    initialVarsDoc,
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

func TestTrigger_ExecuteAPITrigger_Disabled(t *testing.T) {
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
		"description":  "Disabled API trigger",
		"eventsource":  "API_DRIVEN",
		"authtoken":    "token123",
		"enabled":      false,
		"createdat":    time.Now(),
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

	res, err := i.ExecuteAPITrigger(ctx, interfaces.ExecuteAPITriggerParam{
		AuthenticationToken: "token123",
		TriggerID:           tid,
		NotificationURL:     nil,
		Variables: map[string]interface{}{
			"FOO": "bar",
		},
	})

	assert.Error(t, err)
	assert.Nil(t, res)
	assert.Contains(t, err.Error(), "disabled")
}

func TestTrigger_ExecuteTimeDrivenTrigger_Disabled(t *testing.T) {
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
		"description":  "Disabled time trigger",
		"eventsource":  "TIME_DRIVEN",
		"timeinterval": "EVERY_DAY",
		"enabled":      false,
		"createdat":    time.Now(),
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

	res, err := i.ExecuteTimeDrivenTrigger(ctx, interfaces.ExecuteTimeDrivenTriggerParam{
		TriggerID: tid,
	})

	assert.Error(t, err)
	assert.Nil(t, res)
	assert.Contains(t, err.Error(), "disabled")
}

func variablesToSimpleMap(vars []variable.Variable) map[string]interface{} {
	if len(vars) == 0 {
		return nil
	}
	out := make(map[string]interface{}, len(vars))
	for _, v := range vars {
		out[v.Key] = v.Value
	}
	return out
}
