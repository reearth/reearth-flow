package interactor

import (
	"context"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// nilPaddingAssetRepo wraps a real repo.Asset and makes FindByIDs pad
// not-found ids with nil at their requested position, matching the mongo
// implementation's filterAssets (memory.Asset.FindByIDs just omits misses).
type nilPaddingAssetRepo struct {
	repo.Asset
}

func (r *nilPaddingAssetRepo) FindByIDs(ctx context.Context, ids id.AssetIDList) ([]*asset.Asset, error) {
	found, err := r.Asset.FindByIDs(ctx, ids)
	if err != nil {
		return nil, err
	}
	res := make([]*asset.Asset, len(ids))
	for i, aid := range ids {
		for _, a := range found {
			if a != nil && a.ID() == aid {
				res[i] = a
				break
			}
		}
	}
	return res, nil
}

// TestAsset_Fetch_NotFoundFirstElementDoesNotPanic pins a crash observed in
// production on the equivalent Deployment.Fetch: FindByIDs pads a
// not-found/unreadable id with nil, and if that nil lands at index 0,
// res[0].Workspace() panics with a nil pointer dereference. The permission
// check must fall back to the first non-nil element instead.
func TestAsset_Fetch_NotFoundFirstElementDoesNotPanic(t *testing.T) {
	ws := accountsid.NewWorkspaceID()
	assetRepo := &nilPaddingAssetRepo{Asset: memory.NewAsset()}

	a := asset.New().NewID().Workspace(ws).CreatedByUser(accountsid.NewUserID()).Name("f.txt").Size(1).URL("https://example.com/assets/f.txt").NewUUID().MustBuild()
	require.NoError(t, assetRepo.Save(context.Background(), a))

	missingID := id.NewAssetID()

	checker := &recordingChecker{allow: true}
	i := &Asset{repos: &repo.Container{Asset: assetRepo}, permissionChecker: checker}

	require.NotPanics(t, func() {
		res, err := i.Fetch(context.Background(), []id.AssetID{missingID, a.ID()})
		require.NoError(t, err)
		require.Len(t, res, 2)
		assert.Nil(t, res[0], "not-found id stays nil in the result")
		require.NotNil(t, res[1])
		require.Len(t, checker.gotWorkspace, 1)
		assert.Equal(t, ws, checker.gotWorkspace[0], "permission check uses the first non-nil element's workspace")
	})
}

// TestAsset_Fetch_AllNotFound_UsesNoWorkspacePermissionPath pins the other
// half of the same fix: an all-nil batch must take the no-items permission
// path rather than dereferencing a nil element.
func TestAsset_Fetch_AllNotFound_UsesNoWorkspacePermissionPath(t *testing.T) {
	assetRepo := &nilPaddingAssetRepo{Asset: memory.NewAsset()}
	checker := &recordingChecker{allow: true}
	i := &Asset{repos: &repo.Container{Asset: assetRepo}, permissionChecker: checker}

	require.NotPanics(t, func() {
		res, err := i.Fetch(context.Background(), []id.AssetID{id.NewAssetID(), id.NewAssetID()})
		require.NoError(t, err)
		require.Len(t, res, 2)
		assert.Nil(t, res[0])
		assert.Nil(t, res[1])
		assert.Empty(t, checker.gotWorkspace, "no non-nil element means no workspace-scoped check")
	})
}

// TestAsset_Fetch_CrossWorkspaceBatch_DeniedItemOmittedNotLeaked is the
// regression test for the IDOR this fix closes: a batch mixing an asset the
// caller can see with one from a workspace they cannot must return the first
// and nil out the second, instead of using the first item's workspace to
// authorize the whole batch.
func TestAsset_Fetch_CrossWorkspaceBatch_DeniedItemOmittedNotLeaked(t *testing.T) {
	wsAllowed := accountsid.NewWorkspaceID()
	wsDenied := accountsid.NewWorkspaceID()
	assetRepo := &nilPaddingAssetRepo{Asset: memory.NewAsset()}

	own := asset.New().NewID().Workspace(wsAllowed).CreatedByUser(accountsid.NewUserID()).Name("own.txt").Size(1).URL("https://example.com/assets/own.txt").NewUUID().MustBuild()
	require.NoError(t, assetRepo.Save(context.Background(), own))
	victim := asset.New().NewID().Workspace(wsDenied).CreatedByUser(accountsid.NewUserID()).Name("victim.txt").Size(1).URL("https://example.com/assets/victim.txt").NewUUID().MustBuild()
	require.NoError(t, assetRepo.Save(context.Background(), victim))

	checker := &multiWorkspaceChecker{allowed: map[accountsid.WorkspaceID]bool{wsAllowed: true, wsDenied: false}}
	i := &Asset{repos: &repo.Container{Asset: assetRepo}, permissionChecker: checker}

	res, err := i.Fetch(context.Background(), []id.AssetID{own.ID(), victim.ID()})
	require.NoError(t, err)
	require.Len(t, res, 2)
	require.NotNil(t, res[0], "the caller's own asset must still be returned")
	assert.Equal(t, own.ID(), res[0].ID())
	assert.Nil(t, res[1], "the other tenant's asset must not leak just because it rode along with an authorized one")
	assert.Equal(t, 2, checker.calls, "one permission check per distinct workspace in the batch")
}
