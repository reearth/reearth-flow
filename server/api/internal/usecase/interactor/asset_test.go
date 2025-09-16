package interactor

import (
	"bytes"
	"context"
	"io"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/fs"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/reearth/reearthx/account/accountinfrastructure/accountmemory"
	"github.com/reearth/reearthx/appx"
	"github.com/spf13/afero"
	"github.com/stretchr/testify/assert"
)

func TestAsset_Create(t *testing.T) {
	mockAuthInfo := &appx.AuthInfo{
		Token: "token",
	}
	mockUser := user.New().NewID().Name("hoge").Email("abc@bb.cc").MustBuild()

	ctx := context.Background()
	ctx = adapter.AttachAuthInfo(ctx, mockAuthInfo)
	ctx = adapter.AttachReearthxUser(ctx, mockUser)

	// aid := asset.NewID() - removed as the ID is generated in the interactor

	ws := workspace.New().NewID().MustBuild()

	mfs := afero.NewMemMapFs()
	f, _ := fs.NewFile(mfs, "", "")

	mockPermissionCheckerTrue := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
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
		WorkspaceID: ws.ID(),
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
	assert.Equal(t, ws.ID(), res.Workspace())
	assert.Equal(t, "hoge.txt", res.Name())
	assert.Equal(t, uint64(buflen), res.Size())
	assert.Equal(t, "", res.ContentType())
	assert.NotEmpty(t, res.UUID())
	assert.NotEmpty(t, res.URL())

	a, err := uc.repos.Asset.FindByID(ctx, res.ID())
	assert.NoError(t, err)
	assert.Equal(t, res, a)
}
