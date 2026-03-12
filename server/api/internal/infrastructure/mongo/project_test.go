package mongo

import (
	"context"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/mongox/mongotest"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/util"
	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/bson"
)

func TestProject_FindByIDs(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()
	pid := id.NewProjectID()
	pid2 := id.NewProjectID()
	wid := accountsid.NewWorkspaceID()
	wid2 := accountsid.NewWorkspaceID()
	_, _ = c.Collection("project").InsertMany(ctx, []any{
		bson.M{"id": pid.String(), "workspaceid": wid.String()},
		bson.M{"id": pid2.String(), "workspaceid": wid2.String()},
	})

	r := NewProject(mongox.NewClientWithDatabase(c))
	got, err := r.FindByIDs(ctx, id.ProjectIDList{pid})
	assert.NoError(t, err)
	assert.Equal(t, 1, len(got))
	assert.Equal(t, pid, got[0].ID())

	r2 := r.Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{wid2},
	})
	got, err = r2.FindByIDs(ctx, id.ProjectIDList{pid, pid2})
	assert.NoError(t, err)
	assert.Equal(t, 2, len(got))
	assert.Nil(t, got[0])
	assert.Equal(t, pid2, got[1].ID())
}

func TestProject_CountByWorkspace(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()
	wid := accountsid.NewWorkspaceID()
	wid2 := accountsid.NewWorkspaceID()
	_, _ = c.Collection("project").InsertMany(ctx, []any{
		bson.M{"id": "a", "workspaceid": wid.String(), "publishmentstatus": "public"},
		bson.M{"id": "b", "workspaceid": wid.String(), "publishmentstatus": "limited"},
		bson.M{"id": "c", "workspaceid": wid.String()},
		bson.M{"id": "d", "workspaceid": "x", "publishmentstatus": "public"},
	})

	r := NewProject(mongox.NewClientWithDatabase(c))
	got, err := r.CountByWorkspace(ctx, wid)
	assert.Equal(t, 3, got)
	assert.NoError(t, err)

	r2 := r.Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{wid2},
	})
	got, err = r2.CountByWorkspace(ctx, wid)
	assert.Equal(t, repo.ErrOperationDenied, err)
	assert.Zero(t, got)
}

func TestProject_CountPublicByWorkspace(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()
	wid := accountsid.NewWorkspaceID()
	wid2 := accountsid.NewWorkspaceID()
	_, _ = c.Collection("project").InsertMany(ctx, []any{
		bson.M{"id": "a", "workspaceid": wid.String(), "publishmentstatus": "public"},
		bson.M{"id": "b", "workspaceid": wid.String(), "publishmentstatus": "limited"},
		bson.M{"id": "c", "workspaceid": wid.String()},
		bson.M{"id": "d", "workspaceid": "x", "publishmentstatus": "public"},
	})

	r := NewProject(mongox.NewClientWithDatabase(c))
	got, err := r.CountPublicByWorkspace(ctx, wid)
	assert.Equal(t, 2, got)
	assert.NoError(t, err)

	r2 := r.Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{wid2},
	})
	got, err = r2.CountPublicByWorkspace(ctx, wid)
	assert.Equal(t, repo.ErrOperationDenied, err)
	assert.Zero(t, got)
}

func TestProject_FindByPublicName(t *testing.T) {
	now := time.Now().UTC().Truncate(time.Second)
	defer util.MockNow(now)()
	c := mongotest.Connect(t)(t)
	ctx := context.Background()
	wid := accountsid.NewWorkspaceID()
	prj1 := project.New().NewID().Workspace(wid).UpdatedAt(now).MustBuild()
	prj2 := project.New().NewID().Workspace(wid).UpdatedAt(now).MustBuild()
	prj3 := project.New().NewID().Workspace(wid).UpdatedAt(now).MustBuild()
	_, _ = c.Collection("project").InsertMany(ctx, []any{
		util.DR(mongodoc.NewProject(prj1)),
		util.DR(mongodoc.NewProject(prj2)),
		util.DR(mongodoc.NewProject(prj3)),
	})

	r := NewProject(mongox.NewClientWithDatabase(c))

	got, err := r.FindByPublicName(ctx, "alias")
	assert.NoError(t, err)
	assert.Equal(t, prj1, got)

	got, err = r.FindByPublicName(ctx, "aaaaa")
	assert.NoError(t, err)
	assert.Equal(t, prj2, got)

	got, err = r.FindByPublicName(ctx, "alias2")
	assert.Equal(t, rerror.ErrNotFound, err)
	assert.Nil(t, got)

	got, err = r.FindByPublicName(ctx, "bbbbb")
	assert.Equal(t, rerror.ErrNotFound, err)
	assert.Nil(t, got)
}
