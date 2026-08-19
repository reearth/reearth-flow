package interactor

import (
	"context"
	"net/url"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// snapshotAssetRepo hands back the same asset to every FindByID call and just
// counts Deletes, standing in for a transaction retried from a consistent
// snapshot (unlike a mutating in-memory repo, where a second read after the
// first attempt's delete would see the row gone).
type snapshotAssetRepo struct {
	repo.Asset
	a           *asset.Asset
	deleteCalls int
	deleteErr   error
}

func (r *snapshotAssetRepo) FindByID(context.Context, id.AssetID) (*asset.Asset, error) {
	return r.a, nil
}

func (r *snapshotAssetRepo) Delete(context.Context, id.AssetID) error {
	r.deleteCalls++
	if r.deleteErr != nil {
		return r.deleteErr
	}
	return nil
}

// recordingFileGateway records DeleteAsset calls and, at the moment of the
// call, the number of Deletes the asset repo has already committed.
type recordingFileGateway struct {
	gateway.File
	deleteCalls        int
	deleteErr          error
	deleteSeenAtCommit int
	assetRepo          *snapshotAssetRepo
}

func (f *recordingFileGateway) DeleteAsset(context.Context, *url.URL) error {
	f.deleteCalls++
	f.deleteSeenAtCommit = f.assetRepo.deleteCalls
	return f.deleteErr
}

func newDeleteFixture(t *testing.T, tx usecasex.Transactor, deleteErr, fileErr error) (*Asset, *snapshotAssetRepo, *recordingFileGateway, id.AssetID) {
	t.Helper()
	wsID := accountsid.NewWorkspaceID()
	aid := id.NewAssetID()
	a, err := asset.New().ID(aid).Workspace(wsID).CreatedByUser(accountsid.NewUserID()).Name("f.txt").Size(1).URL("https://example.com/assets/f.txt").NewUUID().Build()
	require.NoError(t, err)

	assetRepo := &snapshotAssetRepo{a: a, deleteErr: deleteErr}
	fileGW := &recordingFileGateway{assetRepo: assetRepo, deleteErr: fileErr}

	uc := &Asset{
		repos: &repo.Container{
			Asset:       assetRepo,
			Transaction: tx,
		},
		gateways: &gateway.Container{
			File: fileGW,
		},
		permissionChecker: NewMockPermissionChecker(func(ctx context.Context, resource, action string) (bool, error) {
			return true, nil
		}),
	}
	return uc, assetRepo, fileGW, aid
}

func TestAsset_Delete_ObjectDeletedOnceAfterCommitEvenOnRetry(t *testing.T) {
	tx := &retryingTransactor{}
	uc, assetRepo, fileGW, aid := newDeleteFixture(t, tx, nil, nil)

	_, err := uc.Delete(context.Background(), aid)
	require.NoError(t, err)

	assert.Equal(t, 2, tx.runs, "closure should have run twice via the retrying transactor")
	assert.Equal(t, 2, assetRepo.deleteCalls, "row delete happens on every closure run")
	assert.Equal(t, 1, fileGW.deleteCalls, "object delete must happen exactly once")
	assert.Equal(t, 2, fileGW.deleteSeenAtCommit, "object delete must happen only after the transaction (both runs) committed")
}

func TestAsset_Delete_FailingTransactionNeverDeletesObject(t *testing.T) {
	uc, assetRepo, fileGW, aid := newDeleteFixture(t, nil, rerror.ErrInternalBy(assert.AnError), nil)

	_, err := uc.Delete(context.Background(), aid)
	require.Error(t, err)

	assert.Equal(t, 1, assetRepo.deleteCalls)
	assert.Zero(t, fileGW.deleteCalls, "a failed transaction must never delete the object")
}

func TestAsset_Delete_EmptyURLNeverDeletesObject(t *testing.T) {
	wsID := accountsid.NewWorkspaceID()
	aid := id.NewAssetID()
	a, err := asset.New().ID(aid).Workspace(wsID).CreatedByUser(accountsid.NewUserID()).Name("f.txt").Size(1).URL("").NewUUID().Build()
	require.NoError(t, err)

	assetRepo := &snapshotAssetRepo{a: a}
	fileGW := &recordingFileGateway{assetRepo: assetRepo}
	uc := &Asset{
		repos: &repo.Container{
			Asset: assetRepo,
		},
		gateways: &gateway.Container{
			File: fileGW,
		},
		permissionChecker: NewMockPermissionChecker(func(ctx context.Context, resource, action string) (bool, error) {
			return true, nil
		}),
	}

	_, err = uc.Delete(context.Background(), aid)
	require.NoError(t, err)

	assert.Equal(t, 1, assetRepo.deleteCalls)
	assert.Zero(t, fileGW.deleteCalls, "an asset with an empty URL must never trigger an object delete")
}

func TestAsset_Delete_ObjectDeleteFailureDoesNotFailMutation(t *testing.T) {
	uc, assetRepo, fileGW, aid := newDeleteFixture(t, nil, nil, assert.AnError)

	got, err := uc.Delete(context.Background(), aid)
	require.NoError(t, err, "a best-effort object delete failure must not fail the mutation")
	assert.Equal(t, aid, got)

	assert.Equal(t, 1, assetRepo.deleteCalls)
	assert.Equal(t, 1, fileGW.deleteCalls)
}
