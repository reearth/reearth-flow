package memory

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/projectAccess"
	"github.com/stretchr/testify/assert"
)

func TestNewProjectAccess(t *testing.T) {
	repo := NewProjectAccess()
	assert.NotNil(t, repo)
}

func TestNewProjectAccessWith(t *testing.T) {
	pa, _ := projectAccess.New().
		NewID().
		Project(id.NewProjectID()).
		IsPublic(true).
		Token("token").
		Build()

	repo := NewProjectAccessWith(pa)
	assert.NotNil(t, repo)
}

func TestProjectAccess_FindByProjectID(t *testing.T) {
	pid1 := id.NewProjectID()
	pid2 := id.NewProjectID()
	pa1, _ := projectAccess.New().
		NewID().
		Project(pid1).
		IsPublic(true).
		Token("token").
		Build()
	pa2, _ := projectAccess.New().
		NewID().
		Project(pid2).
		IsPublic(true).
		Token("token").
		Build()
	repo := NewProjectAccessWith(pa1, pa2)

	ctx := context.Background()
	out, err := repo.FindByProjectID(ctx, pid1)
	assert.NoError(t, err)
	assert.Equal(t, pa1, out)

	out, err = repo.FindByProjectID(ctx, id.NewProjectID())
	assert.Error(t, err)
	assert.Nil(t, out)
}

func TestProjectAccess_FindByToken(t *testing.T) {
	pa1, _ := projectAccess.New().
		NewID().
		Project(id.NewProjectID()).
		IsPublic(true).
		Token("token1").
		Build()
	pa2, _ := projectAccess.New().
		NewID().
		Project(id.NewProjectID()).
		IsPublic(true).
		Token("token2").
		Build()
	repo := NewProjectAccessWith(pa1, pa2)

	ctx := context.Background()
	out, err := repo.FindByToken(ctx, "token1")
	assert.NoError(t, err)
	assert.Equal(t, pa1, out)

	out, err = repo.FindByToken(ctx, "not-exist-token")
	assert.Error(t, err)
	assert.Nil(t, out)
}

func TestProjectAccess_Save(t *testing.T) {
	pid := id.NewProjectID()
	pa, _ := projectAccess.New().
		NewID().
		Project(pid).
		IsPublic(true).
		Token("token").
		Build()
	repo := NewProjectAccess()

	ctx := context.Background()
	err := repo.Save(ctx, pa)
	assert.NoError(t, err)

	out, err := repo.FindByProjectID(ctx, pid)
	assert.NoError(t, err)
	assert.Equal(t, pa, out)
}
