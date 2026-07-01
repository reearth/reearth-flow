package postgres_test

import (
	"context"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/pgxx"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newAsset(t *testing.T, wid accountsid.WorkspaceID, aid id.AssetID, name string, size uint64) *asset.Asset {
	t.Helper()
	uid := accountsid.NewUserID()
	a, err := asset.New().
		ID(aid).
		Workspace(wid).
		CreatedAt(time.Now()).
		Name(name).
		FileName(name + ".txt").
		Size(size).
		URL("https://example.com/" + name).
		ContentType("text/plain").
		UUID("uuid-" + name).
		FlatFiles(false).
		Public(false).
		CreatedByUser(uid).
		Build()
	require.NoError(t, err)
	return a
}

func TestAsset_Save_FindByID(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewAsset(pgxx.NewClient(pool))

	wid := accountsid.NewWorkspaceID()
	aid := id.NewAssetID()
	a := newAsset(t, wid, aid, "myfile", 1234)

	require.NoError(t, r.Save(ctx, a))
	got, err := r.FindByID(ctx, aid)
	require.NoError(t, err)
	assert.Equal(t, aid, got.ID())
	assert.Equal(t, wid, got.Workspace())
	assert.Equal(t, "myfile", got.Name())
	assert.Equal(t, uint64(1234), got.Size())
}

func TestAsset_Save_FindByID_WithOptionals(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewAsset(pgxx.NewClient(pool))

	wid := accountsid.NewWorkspaceID()
	aid := id.NewAssetID()
	pid := id.NewProjectID()
	uid := accountsid.NewUserID()
	tid := id.NewThreadID()

	a, err := asset.New().
		ID(aid).
		Workspace(wid).
		CreatedAt(time.Now()).
		Name("withoptionals").
		FileName("withoptionals.txt").
		Size(500).
		URL("https://example.com/withoptionals").
		ContentType("text/plain").
		UUID("uuid-withoptionals").
		FlatFiles(true).
		Public(true).
		Project(pid).
		CreatedByUser(uid).
		Thread(&tid).
		ArchiveExtractionStatus(asset.ArchiveExtractionStatusDone).
		Build()
	require.NoError(t, err)

	require.NoError(t, r.Save(ctx, a))
	got, err := r.FindByID(ctx, aid)
	require.NoError(t, err)
	assert.Equal(t, aid, got.ID())
	assert.Equal(t, pid, got.Project())
	assert.Equal(t, &uid, got.User())
	assert.Equal(t, &tid, got.Thread())
	assert.NotNil(t, got.ArchiveExtractionStatus())
	assert.Equal(t, asset.ArchiveExtractionStatusDone, *got.ArchiveExtractionStatus())
	assert.True(t, got.FlatFiles())
	assert.True(t, got.Public())
}

func TestAsset_FindByID_NotFound(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	got, err := postgres.NewAsset(pgxx.NewClient(pool)).FindByID(context.Background(), id.NewAssetID())
	assert.Error(t, err)
	assert.Nil(t, got)
}

func TestAsset_FindByIDs_OrderAndMissing(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewAsset(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()

	aid1 := id.NewAssetID()
	aid2 := id.NewAssetID()
	missing := id.NewAssetID()

	require.NoError(t, r.Save(ctx, newAsset(t, wid, aid1, "first", 100)))
	require.NoError(t, r.Save(ctx, newAsset(t, wid, aid2, "second", 200)))

	// pgxx.OrderByIDs drops missing IDs (no nil padding); result has 2 items in request order
	got, err := r.FindByIDs(ctx, id.AssetIDList{aid2, missing, aid1})
	require.NoError(t, err)
	require.Len(t, got, 2)
	assert.Equal(t, aid2, got[0].ID())
	assert.Equal(t, aid1, got[1].ID())
}

func TestAsset_FindByWorkspace_NoPagination(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewAsset(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	wid2 := accountsid.NewWorkspaceID()

	require.NoError(t, r.Save(ctx, newAsset(t, wid, id.NewAssetID(), "a1", 100)))
	require.NoError(t, r.Save(ctx, newAsset(t, wid, id.NewAssetID(), "a2", 200)))
	require.NoError(t, r.Save(ctx, newAsset(t, wid2, id.NewAssetID(), "other", 300)))

	got, info, err := r.FindByWorkspace(ctx, wid, repo.AssetFilter{})
	require.NoError(t, err)
	require.NotNil(t, info)
	assert.Len(t, got, 2)
}

func TestAsset_FindByWorkspace_Paginated(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewAsset(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()

	for i := 0; i < 5; i++ {
		require.NoError(t, r.Save(ctx, newAsset(t, wid, id.NewAssetID(), "asset", 100)))
	}

	page := &interfaces.PaginationParam{Page: &interfaces.PageBasedPaginationParam{Page: 1, PageSize: 2}}
	got, info, err := r.FindByWorkspace(ctx, wid, repo.AssetFilter{Pagination: page})
	require.NoError(t, err)
	assert.Len(t, got, 2)
	assert.Equal(t, int64(5), info.TotalCount)
	assert.Equal(t, 3, info.TotalPages)
	assert.Equal(t, 1, info.CurrentPage)
}

func TestAsset_FindByWorkspace_Paginated_DefaultsToDescWhenOrderDirNil(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewAsset(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()

	require.NoError(t, r.Save(ctx, newAsset(t, wid, id.NewAssetID(), "alpha", 100)))
	require.NoError(t, r.Save(ctx, newAsset(t, wid, id.NewAssetID(), "zulu", 200)))

	orderBy := "name"
	page := &interfaces.PaginationParam{
		Page: &interfaces.PageBasedPaginationParam{
			Page:     1,
			PageSize: 2,
			OrderBy:  &orderBy,
		},
	}

	got, _, err := r.FindByWorkspace(ctx, wid, repo.AssetFilter{Pagination: page})
	require.NoError(t, err)
	require.Len(t, got, 2)
	assert.Equal(t, "zulu", got[0].Name())
	assert.Equal(t, "alpha", got[1].Name())
}

func TestAsset_FindByWorkspace_Keyword(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewAsset(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()

	require.NoError(t, r.Save(ctx, newAsset(t, wid, id.NewAssetID(), "findme-special", 100)))
	require.NoError(t, r.Save(ctx, newAsset(t, wid, id.NewAssetID(), "other-file", 200)))

	kw := "findme"
	got, _, err := r.FindByWorkspace(ctx, wid, repo.AssetFilter{Keyword: &kw})
	require.NoError(t, err)
	assert.Len(t, got, 1)
	assert.Equal(t, "findme-special", got[0].Name())
}

func TestAsset_FindByWorkspace_Sort(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewAsset(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()

	require.NoError(t, r.Save(ctx, newAsset(t, wid, id.NewAssetID(), "small", 100)))
	require.NoError(t, r.Save(ctx, newAsset(t, wid, id.NewAssetID(), "large", 9999)))

	sortBySize := asset.SortTypeSIZE
	got, _, err := r.FindByWorkspace(ctx, wid, repo.AssetFilter{Sort: &sortBySize})
	require.NoError(t, err)
	require.Len(t, got, 2)
	// ASC order: small first
	assert.Equal(t, uint64(100), got[0].Size())
	assert.Equal(t, uint64(9999), got[1].Size())
}

func TestAsset_TotalSizeByWorkspace(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewAsset(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()

	require.NoError(t, r.Save(ctx, newAsset(t, wid, id.NewAssetID(), "f1", 1000)))
	require.NoError(t, r.Save(ctx, newAsset(t, wid, id.NewAssetID(), "f2", 2500)))

	total, err := r.TotalSizeByWorkspace(ctx, wid)
	require.NoError(t, err)
	assert.Equal(t, uint64(3500), total)
}

func TestAsset_TotalSizeByWorkspace_Empty(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewAsset(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()

	total, err := r.TotalSizeByWorkspace(ctx, wid)
	require.NoError(t, err)
	assert.Equal(t, uint64(0), total)
}

func TestAsset_Delete(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewAsset(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	aid := id.NewAssetID()

	require.NoError(t, r.Save(ctx, newAsset(t, wid, aid, "todelete", 100)))
	require.NoError(t, r.Delete(ctx, aid))

	got, err := r.FindByID(ctx, aid)
	assert.Error(t, err)
	assert.Nil(t, got)
}

func TestAsset_Delete_WithWorkspaceFilter(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	base := postgres.NewAsset(pgxx.NewClient(pool))
	wid1 := accountsid.NewWorkspaceID()
	wid2 := accountsid.NewWorkspaceID()
	aid1 := id.NewAssetID()
	aid2 := id.NewAssetID()

	require.NoError(t, base.Save(ctx, newAsset(t, wid1, aid1, "in-scope", 100)))
	require.NoError(t, base.Save(ctx, newAsset(t, wid2, aid2, "out-of-scope", 200)))

	r := base.Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{wid1},
		Writable: accountsid.WorkspaceIDList{wid1},
	})

	require.NoError(t, r.Delete(ctx, aid1))
	require.NoError(t, r.Delete(ctx, aid2)) // not writable → no-op

	// aid2 should still exist
	got, err := base.FindByID(ctx, aid2)
	require.NoError(t, err)
	assert.NotNil(t, got)
}

func TestAsset_Filtered_CanReadDenied(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	wid := accountsid.NewWorkspaceID()
	r := postgres.NewAsset(pgxx.NewClient(pool)).Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{accountsid.NewWorkspaceID()},
	})
	got, info, err := r.FindByWorkspace(ctx, wid, repo.AssetFilter{})
	require.NoError(t, err)
	assert.Empty(t, got)
	assert.NotNil(t, info)
}

func TestAsset_Save_CanWriteDenied(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	wid := accountsid.NewWorkspaceID()
	r := postgres.NewAsset(pgxx.NewClient(pool)).Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{wid},
		Writable: accountsid.WorkspaceIDList{accountsid.NewWorkspaceID()},
	})
	a := newAsset(t, wid, id.NewAssetID(), "denied", 100)
	err := r.Save(ctx, a)
	assert.ErrorIs(t, err, repo.ErrOperationDenied)
}
