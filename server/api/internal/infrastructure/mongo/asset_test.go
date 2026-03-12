package mongo

import (
	"context"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/asset"
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
					Workspace(accountsid.NewWorkspaceID()).
					CreatedByUser(accountsid.NewUserID()).
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

func TestFindByWorkspace(t *testing.T) {
	init := mongotest.Connect(t)

	type sortCase struct {
		name      string
		orderBy   string
		orderDir  string
		wantNames []string
	}

	tests := []sortCase{
		{
			name:      "name ASC",
			orderBy:   "name",
			orderDir:  "ASC",
			wantNames: []string{"aaa", "bbb", "ccc"},
		},
		{
			name:      "name DESC",
			orderBy:   "name",
			orderDir:  "DESC",
			wantNames: []string{"ccc", "bbb", "aaa"},
		},
		{
			name:      "size ASC",
			orderBy:   "size",
			orderDir:  "ASC",
			wantNames: []string{"bbb", "ccc", "aaa"}, // size: 1(bbb), 10(ccc), 100(aaa)
		},
		{
			name:      "size DESC",
			orderBy:   "size",
			orderDir:  "DESC",
			wantNames: []string{"aaa", "ccc", "bbb"},
		},
		{
			name:      "createdAt ASC",
			orderBy:   "createdAt",
			orderDir:  "ASC",
			wantNames: []string{"ccc", "bbb", "aaa"}, // createdAt: old(ccc), mid(bbb), new(aaa)
		},
		{
			name:      "createdAt DESC",
			orderBy:   "createdAt",
			orderDir:  "DESC",
			wantNames: []string{"aaa", "bbb", "ccc"},
		},
	}

	for _, tc := range tests {
		tc := tc

		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()

			client := init(t)
			repoAsset := NewAsset(mongox.NewClientWithDatabase(client))
			ctx := context.Background()

			wid := accountsid.NewWorkspaceID()
			uid := accountsid.NewUserID()

			aAaa := asset.New().
				NewID().
				CreatedAt(time.Date(2025, 1, 3, 0, 0, 0, 0, time.UTC)).
				Workspace(wid).
				CreatedByUser(uid).
				FileName("aaa.json").
				Name("aaa").
				Size(100).
				URL("https://example.com/aaa").
				ContentType("application/json").
				NewUUID().
				MustBuild()

			aBbb := asset.New().
				NewID().
				CreatedAt(time.Date(2025, 1, 2, 0, 0, 0, 0, time.UTC)).
				Workspace(wid).
				CreatedByUser(uid).
				FileName("bbb.json").
				Name("bbb").
				Size(1).
				URL("https://example.com/bbb").
				ContentType("application/json").
				NewUUID().
				MustBuild()

			aCcc := asset.New().
				NewID().
				CreatedAt(time.Date(2025, 1, 1, 0, 0, 0, 0, time.UTC)).
				Workspace(wid).
				CreatedByUser(uid).
				FileName("ccc.json").
				Name("ccc").
				Size(10).
				URL("https://example.com/ccc").
				ContentType("application/json").
				NewUUID().
				MustBuild()

			assert.NoError(t, repoAsset.Save(ctx, aBbb))
			assert.NoError(t, repoAsset.Save(ctx, aAaa))
			assert.NoError(t, repoAsset.Save(ctx, aCcc))

			p := &interfaces.PaginationParam{
				Page: &interfaces.PageBasedPaginationParam{
					Page:     1,
					PageSize: 30,
					OrderBy:  &tc.orderBy,
					OrderDir: &tc.orderDir,
				},
			}

			got, _, err := repoAsset.FindByWorkspace(ctx, wid, repo.AssetFilter{
				Sort:       nil,
				Keyword:    nil,
				Pagination: p,
			})
			assert.NoError(t, err)

			if assert.Len(t, got, 3) {
				assert.Equal(t, tc.wantNames[0], got[0].Name())
				assert.Equal(t, tc.wantNames[1], got[1].Name())
				assert.Equal(t, tc.wantNames[2], got[2].Name())
			}
		})
	}
}

// Removed TestAsset_TotalSizeByWorkspace as we switched to project-based assets
