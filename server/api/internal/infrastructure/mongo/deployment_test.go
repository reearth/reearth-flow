package mongo

import (
	"context"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/mongox/mongotest"
	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/bson"
)

func TestDeployment_FindByIDs(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()

	did1 := id.NewDeploymentID()
	did2 := id.NewDeploymentID()
	wid := accountsid.NewWorkspaceID()
	wid2 := accountsid.NewWorkspaceID()

	_, _ = c.Collection("deployment").InsertMany(ctx, []any{
		bson.M{"id": did1.String(), "workspaceid": wid.String()},
		bson.M{"id": did2.String(), "workspaceid": wid2.String()},
	})

	r := NewDeployment(mongox.NewClientWithDatabase(c))

	got, err := r.FindByIDs(ctx, id.DeploymentIDList{did1})
	assert.NoError(t, err)
	assert.Equal(t, 1, len(got))
	assert.Equal(t, did1, got[0].ID())

	r2 := r.Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{wid2},
	})
	got, err = r2.FindByIDs(ctx, id.DeploymentIDList{did1, did2})
	assert.NoError(t, err)
	assert.Equal(t, 2, len(got))
	assert.Nil(t, got[0])
	assert.Equal(t, did2, got[1].ID())
}

func TestDeployment_FindByWorkspace(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()

	wid := accountsid.NewWorkspaceID()
	wid2 := accountsid.NewWorkspaceID()

	_, _ = c.Collection("deployment").InsertMany(ctx, []any{
		bson.M{"id": "d1", "workspaceid": wid.String(), "version": "v1", "updatedat": time.Now().Add(-2 * time.Hour)},
		bson.M{"id": "d2", "workspaceid": wid.String(), "version": "v2", "updatedat": time.Now().Add(-1 * time.Hour)},
		bson.M{"id": "d3", "workspaceid": wid.String(), "version": "v3", "updatedat": time.Now()},
		bson.M{"id": "d4", "workspaceid": wid2.String(), "version": "v1", "updatedat": time.Now()},
	})

	r := NewDeployment(mongox.NewClientWithDatabase(c))

	// Test without pagination
	got, pageInfo, err := r.FindByWorkspace(ctx, wid, nil, nil)
	assert.NoError(t, err)
	assert.NotNil(t, pageInfo)
	assert.Equal(t, 3, len(got))
	assert.Equal(t, "d3", got[0].ID().String())
	assert.Equal(t, "d2", got[1].ID().String())
	assert.Equal(t, "d1", got[2].ID().String())
	assert.Equal(t, int64(3), pageInfo.TotalCount)
	assert.Equal(t, 1, pageInfo.CurrentPage)
	assert.Equal(t, 1, pageInfo.TotalPages)

	// Test page-based pagination: first page
	pagination := &interfaces.PaginationParam{
		Page: &interfaces.PageBasedPaginationParam{
			Page:     1,
			PageSize: 2,
		},
	}
	got, pageInfo, err = r.FindByWorkspace(ctx, wid, pagination, nil)
	assert.NoError(t, err)
	assert.NotNil(t, pageInfo)
	assert.Equal(t, 2, len(got))
	assert.Equal(t, "d3", got[0].ID().String())
	assert.Equal(t, "d2", got[1].ID().String())
	assert.Equal(t, int64(3), pageInfo.TotalCount)
	assert.Equal(t, 1, pageInfo.CurrentPage)
	assert.Equal(t, 2, pageInfo.TotalPages)

	// Test page-based pagination: second page
	pagination = &interfaces.PaginationParam{
		Page: &interfaces.PageBasedPaginationParam{
			Page:     2,
			PageSize: 2,
		},
	}
	got, pageInfo, err = r.FindByWorkspace(ctx, wid, pagination, nil)
	assert.NoError(t, err)
	assert.NotNil(t, pageInfo)
	assert.Equal(t, 1, len(got))
	assert.Equal(t, "d1", got[0].ID().String())
	assert.Equal(t, int64(3), pageInfo.TotalCount)
	assert.Equal(t, 2, pageInfo.CurrentPage)
	assert.Equal(t, 2, pageInfo.TotalPages)

	// Test page-based pagination: empty page
	pagination = &interfaces.PaginationParam{
		Page: &interfaces.PageBasedPaginationParam{
			Page:     3,
			PageSize: 2,
		},
	}
	got, pageInfo, err = r.FindByWorkspace(ctx, wid, pagination, nil)
	assert.NoError(t, err)
	assert.NotNil(t, pageInfo)
	assert.Equal(t, 0, len(got))
	assert.Equal(t, int64(3), pageInfo.TotalCount)
	assert.Equal(t, 3, pageInfo.CurrentPage)
	assert.Equal(t, 2, pageInfo.TotalPages)

	// Test with workspace filter
	r2 := r.Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{wid2},
	})
	got, pageInfo, err = r2.FindByWorkspace(ctx, wid, nil, nil)
	assert.NoError(t, err)
	assert.NotNil(t, pageInfo)
	assert.Equal(t, 0, len(got))
	assert.Equal(t, int64(0), pageInfo.TotalCount)
	assert.Equal(t, 1, pageInfo.CurrentPage)
	assert.Equal(t, 1, pageInfo.TotalPages)
}

func TestDeployment_FindByVersion(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()

	wid := accountsid.NewWorkspaceID()
	wid2 := accountsid.NewWorkspaceID()

	_, _ = c.Collection("deployment").InsertMany(ctx, []any{
		bson.M{"id": "d1", "workspaceid": wid.String(), "version": "v1"},
		bson.M{"id": "d2", "workspaceid": wid2.String(), "version": "v2"},
	})

	r := NewDeployment(mongox.NewClientWithDatabase(c))

	got, err := r.FindByVersion(ctx, wid, nil, "v1")
	assert.NoError(t, err)
	assert.Equal(t, "d1", got.ID().String())

	got, err = r.FindByVersion(ctx, wid, nil, "v2")
	assert.Error(t, err)
	assert.Nil(t, got)
}

func TestDeployment_FindHead(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()

	wid := accountsid.NewWorkspaceID()
	wid2 := accountsid.NewWorkspaceID()

	_, _ = c.Collection("deployment").InsertMany(ctx, []any{
		bson.M{"id": "d1", "workspaceid": wid.String(), "ishead": true},
		bson.M{"id": "d2", "workspaceid": wid2.String(), "ishead": false},
	})

	r := NewDeployment(mongox.NewClientWithDatabase(c))

	got, err := r.FindHead(ctx, wid, nil)
	assert.NoError(t, err)
	assert.Equal(t, "d1", got.ID().String())

	got, err = r.FindHead(ctx, wid2, nil)
	assert.Error(t, err)
	assert.Nil(t, got)
}

func TestDeployment_Save(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()

	wid := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()

	dep := deployment.New().
		ID(did).
		Workspace(wid).
		Version("v1").
		UpdatedAt(time.Now()).
		MustBuild()

	r := NewDeployment(mongox.NewClientWithDatabase(c))

	err := r.Save(ctx, dep)
	assert.NoError(t, err)

	got, err := r.FindByID(ctx, did)
	assert.NoError(t, err)
	assert.Equal(t, dep.ID(), got.ID())
	assert.Equal(t, dep.Version(), got.Version())
}

func TestDeployment_Remove(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()

	did := id.NewDeploymentID()

	_, _ = c.Collection("deployment").InsertOne(ctx, bson.M{"id": did.String()})

	r := NewDeployment(mongox.NewClientWithDatabase(c))

	err := r.Remove(ctx, did)
	assert.NoError(t, err)

	got, err := r.FindByID(ctx, did)
	assert.Error(t, err)
	assert.Nil(t, got)
}
