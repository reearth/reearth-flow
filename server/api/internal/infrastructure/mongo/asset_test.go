package mongo

import (
	"context"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/mongox/mongotest"
	"github.com/stretchr/testify/assert"
)

func TestFindByID(t *testing.T) {
	tests := []struct {
		Name     string
		Expected struct {
			Name  string
			Asset *asset.Asset
		}
	}{
		{
			Expected: struct {
				Name  string
				Asset *asset.Asset
			}{
				Asset: asset.New().
					NewID().
					CreatedAt(time.Now()).
					Workspace(accountdomain.NewWorkspaceID()).
					CreatedByUser(accountdomain.NewUserID()).
					FileName("file.json").
					Name("name").
					Size(10).
					URL("hxxps://https://reearth.io/").
					ContentType("json").
					NewUUID().
					MustBuild(),
			},
		},
	}

	init := mongotest.Connect(t)

	for _, tc := range tests {
		tc := tc

		t.Run(tc.Name, func(t *testing.T) {
			t.Parallel()

			client := init(t)

			repo := NewAsset(mongox.NewClientWithDatabase(client))
			ctx := context.Background()
			err := repo.Save(ctx, tc.Expected.Asset)
			assert.NoError(t, err)

			got, err := repo.FindByID(ctx, tc.Expected.Asset.ID())
			assert.NoError(t, err)
			assert.Equal(t, tc.Expected.Asset.ID(), got.ID())
			assert.Equal(t, tc.Expected.Asset.CreatedAt(), got.CreatedAt())
			assert.Equal(t, tc.Expected.Asset.Workspace(), got.Workspace())
			assert.Equal(t, tc.Expected.Asset.URL(), got.URL())
			assert.Equal(t, tc.Expected.Asset.Size(), got.Size())
			assert.Equal(t, tc.Expected.Asset.Name(), got.Name())
			assert.Equal(t, tc.Expected.Asset.ContentType(), got.ContentType())
		})
	}
}

// Removed TestAsset_TotalSizeByWorkspace as we switched to project-based assets
