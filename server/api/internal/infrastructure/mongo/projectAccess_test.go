package mongo

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/id"

	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/mongox/mongotest"
	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/bson"
)

func TestProjectAccess_FindByProjectID(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()
	paid1 := id.NewProjectAccessID()
	paid2 := id.NewProjectAccessID()
	pid1 := id.NewProjectID()
	pid2 := id.NewProjectID()

	_, _ = c.Collection("projectAccess").InsertMany(ctx, []any{
		bson.M{"id": paid1.String(), "projectid": pid1.String(), "ispublic": true, "token": "token"},
		bson.M{"id": paid2.String(), "projectid": pid2.String(), "ispublic": true, "token": "token"},
	})

	pa := NewProjectAccess(mongox.NewClientWithDatabase(c))

	got, err := pa.FindByProjectID(ctx, pid1)
	assert.NoError(t, err)
	assert.Equal(t, paid1, got.ID())
	assert.Equal(t, pid1, got.Project())
	assert.True(t, got.IsPublic())
	assert.Equal(t, "token", got.Token())

	got, err = pa.FindByProjectID(ctx, id.NewProjectID())
	assert.Error(t, err)
	assert.Nil(t, got)
}

func TestProjectAccess_FindByToken(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()
	paid1 := id.NewProjectAccessID()
	paid2 := id.NewProjectAccessID()
	pid1 := id.NewProjectID()
	pid2 := id.NewProjectID()

	_, _ = c.Collection("projectAccess").InsertMany(ctx, []any{
		bson.M{"id": paid1.String(), "projectid": pid1.String(), "ispublic": true, "token": "token1"},
		bson.M{"id": paid2.String(), "projectid": pid2.String(), "ispublic": true, "token": "token2"},
	})

	pa := NewProjectAccess(mongox.NewClientWithDatabase(c))

	got, err := pa.FindByToken(ctx, "token1")
	assert.NoError(t, err)
	assert.Equal(t, paid1, got.ID())
	assert.Equal(t, pid1, got.Project())
	assert.True(t, got.IsPublic())
	assert.Equal(t, "token1", got.Token())

	got, err = pa.FindByToken(ctx, "not-exist-token")
	assert.Error(t, err)
	assert.Nil(t, got)
}

func TestProjectAccess_Save(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()
	paid := id.NewProjectAccessID()
	pid := id.NewProjectID()

	_, _ = c.Collection("projectAccess").InsertOne(ctx, bson.M{"id": paid.String(), "projectid": pid.String(), "ispublic": true, "token": "token"})

	pa := NewProjectAccess(mongox.NewClientWithDatabase(c))

	got, err := pa.FindByProjectID(ctx, pid)
	assert.NoError(t, err)
	assert.Equal(t, paid, got.ID())
	assert.Equal(t, pid, got.Project())
	assert.True(t, got.IsPublic())
	assert.Equal(t, "token", got.Token())

	got.SetIsPublic(false)
	got.SetToken("")
	err = pa.Save(ctx, got)
	assert.NoError(t, err)

	got, err = pa.FindByProjectID(ctx, pid)
	assert.NoError(t, err)
	assert.Equal(t, paid, got.ID())
	assert.Equal(t, pid, got.Project())
	assert.False(t, got.IsPublic())
	assert.Empty(t, got.Token())
}
