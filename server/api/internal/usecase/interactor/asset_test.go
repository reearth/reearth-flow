package interactor

import (
	"bytes"
	"context"
	"io"
	"os"
	"testing"
	"time"

	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/fs"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/file"
	reearthxworkspace "github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/reearth/reearthx/account/accountinfrastructure/accountmemory"
	"github.com/reearth/reearthx/appx"
	"github.com/spf13/afero"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestAsset_Create(t *testing.T) {
	mockAuthInfo := &appx.AuthInfo{
		Token: "token",
	}
	mockUser := accountsuser.New().NewID().Name("hoge").Email("abc@bb.cc").MustBuild()

	ctx := context.Background()
	ctx = adapter.AttachAuthInfo(ctx, mockAuthInfo)
	ctx = adapter.AttachUser(ctx, mockUser)

	// aid := asset.NewID() - removed as the ID is generated in the interactor

	ws := reearthxworkspace.New().NewID().MustBuild()

	mfs := afero.NewMemMapFs()
	f, _ := fs.NewFile(mfs, "", "")

	mockPermissionCheckerTrue := NewMockPermissionChecker(func(ctx context.Context, resource, action string) (bool, error) {
		return true, nil
	})

	uc := &Asset{
		repos: &repo.Container{
			Asset:     memory.NewAsset(),
			Workspace: accountmemory.NewWorkspaceWith(ws),
		},
		gateways: &gateway.Container{
			File: f,
		},
		permissionChecker: mockPermissionCheckerTrue,
	}

	buf := bytes.NewBufferString("Hello")
	buflen := int64(buf.Len())
	res, err := uc.Create(ctx, interfaces.CreateAssetParam{
		WorkspaceID: accountsworkspace.ID(ws.ID()),
		File: &file.File{
			Content:     io.NopCloser(buf),
			Path:        "hoge.txt",
			ContentType: "",
			Size:        buflen,
		},
	})
	assert.NoError(t, err)
	assert.NotNil(t, res)
	assert.NotEmpty(t, res.ID())
	assert.Equal(t, accountsworkspace.ID(ws.ID()), res.Workspace())
	assert.Equal(t, "hoge.txt", res.Name())
	assert.Equal(t, uint64(buflen), res.Size())
	assert.Equal(t, "", res.ContentType())
	assert.NotEmpty(t, res.UUID())
	assert.NotEmpty(t, res.URL())

	a, err := uc.repos.Asset.FindByID(ctx, res.ID())
	assert.NoError(t, err)
	assert.Equal(t, res, a)
}

func TestAsset_Create_RefusesAnAssetOverThePlansLimit(t *testing.T) {
	t.Parallel()

	ctx := adapter.AttachAuthInfo(context.Background(), &appx.AuthInfo{Token: "token"})
	ctx = adapter.AttachUser(ctx, accountsuser.New().NewID().Name("hoge").Email("abc@bb.cc").MustBuild())
	ws := reearthxworkspace.New().NewID().MustBuild()
	// newUC builds a fixture of its own for each subtest, with the in-memory
	// storage its uploads land in.
	newUC := func(t *testing.T) (*Asset, afero.Fs) {
		t.Helper()
		mfs := afero.NewMemMapFs()
		f, err := fs.NewFile(mfs, "https://example.com/assets", "")
		require.NoError(t, err)
		return &Asset{
			repos:             &repo.Container{Asset: memory.NewAsset(), Workspace: accountmemory.NewWorkspaceWith(ws)},
			gateways:          &gateway.Container{File: f},
			permissionChecker: NewMockPermissionChecker(func(context.Context, string, string) (bool, error) { return true, nil }),
			planLimits:        planLimits{policyChecker: &planChecker{}, now: time.Now},
		}, mfs
	}
	tooLarge := func(declared int64) *file.File {
		return &file.File{Content: io.NopCloser(bytes.NewReader(make([]byte, 5<<20+1))), Path: "big.bin", Size: declared}
	}

	for name, declared := range map[string]int64{"a declared size": 5<<20 + 1, "an undeclared size": 0} {
		t.Run(name, func(t *testing.T) {
			t.Parallel()

			uc, mfs := newUC(t)

			_, err := uc.Create(ctx, interfaces.CreateAssetParam{WorkspaceID: accountsworkspace.ID(ws.ID()), File: tooLarge(declared)})

			assert.ErrorIs(t, err, interfaces.ErrAssetTooLarge)
			assert.ErrorContains(t, err, "5MB")
			saved, _, err := uc.repos.Asset.FindByWorkspace(ctx, accountsworkspace.ID(ws.ID()), repo.AssetFilter{})
			assert.NoError(t, err)
			assert.Empty(t, saved)
			// An upload refused only after it was stored is removed again.
			stored, err := afero.ReadDir(mfs, "assets")
			if err == nil {
				assert.Empty(t, stored, "the refused upload must not stay in storage")
			} else {
				assert.True(t, os.IsNotExist(err), err)
			}
		})
	}

	t.Run("an upload link for a declared size over the limit", func(t *testing.T) {
		t.Parallel()

		uc, _ := newUC(t)

		_, err := uc.CreateUpload(ctx, interfaces.CreateAssetUploadParam{
			WorkspaceID: accountsworkspace.ID(ws.ID()), Filename: "big.bin", ContentLength: 5<<20 + 1,
		})

		assert.ErrorIs(t, err, interfaces.ErrAssetTooLarge)
	})
}
