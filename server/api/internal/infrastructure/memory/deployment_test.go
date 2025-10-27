package memory

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
)

func TestDeployment(t *testing.T) {
	ctx := context.Background()
	wsID, _ := id.WorkspaceIDFrom("workspace1")
	projectID := id.NewProjectID()
	deploymentID := id.NewDeploymentID()
	version := "v1"

	d := deployment.New().
		ID(deploymentID).
		Workspace(wsID).
		Project(&projectID).
		Version(version).
		IsHead(true).
		MustBuild()

	repo := NewDeployment()

	t.Run("Save and FindByID", func(t *testing.T) {
		err := repo.Save(ctx, d)
		assert.NoError(t, err)

		result, err := repo.FindByID(ctx, deploymentID)
		assert.NoError(t, err)
		assert.Equal(t, d, result)
	})

	t.Run("FindByWorkspace", func(t *testing.T) {
		results, pageInfo, err := repo.FindByWorkspace(ctx, wsID, nil, nil)
		assert.NoError(t, err)
		assert.NotNil(t, pageInfo)
		assert.Len(t, results, 1)
		assert.Equal(t, d, results[0])
	})

	t.Run("FindByProject", func(t *testing.T) {
		result, err := repo.FindByProject(ctx, projectID)
		assert.NoError(t, err)
		assert.Equal(t, d, result)
	})

	t.Run("FindByIDs", func(t *testing.T) {
		results, err := repo.FindByIDs(ctx, id.DeploymentIDList{deploymentID})
		assert.NoError(t, err)
		assert.Len(t, results, 1)
		assert.Equal(t, d, results[0])
	})

	t.Run("FindByVersion", func(t *testing.T) {
		result, err := repo.FindByVersion(ctx, wsID, &projectID, version)
		assert.NoError(t, err)
		assert.Equal(t, d, result)
	})

	t.Run("FindHead", func(t *testing.T) {
		d.SetIsHead(true)
		err := repo.Save(ctx, d)
		assert.NoError(t, err)

		result, err := repo.FindHead(ctx, wsID, &projectID)
		assert.NoError(t, err)
		assert.Equal(t, d, result)
	})

	t.Run("FindVersions", func(t *testing.T) {
		d2 := deployment.New().
			ID(id.NewDeploymentID()).
			Workspace(wsID).
			Project(&projectID).
			Version("v2").
			MustBuild()

		err := repo.Save(ctx, d2)
		assert.NoError(t, err)

		results, err := repo.FindVersions(ctx, wsID, &projectID)
		assert.NoError(t, err)
		assert.Len(t, results, 2)
		assert.Equal(t, "v2", results[0].Version())
		assert.Equal(t, "v1", results[1].Version())
	})

	t.Run("Remove", func(t *testing.T) {
		err := repo.Remove(ctx, deploymentID)
		assert.NoError(t, err)

		_, err = repo.FindByID(ctx, deploymentID)
		assert.Error(t, err)
		assert.Equal(t, "not found", err.Error())
	})
}

func TestDeployment_FindByWorkspace(t *testing.T) {
	ctx := context.Background()
	wsID := id.NewWorkspaceID()
	wsID2 := id.NewWorkspaceID()

	// Create test data
	d1 := deployment.New().NewID().Workspace(wsID).Version("v1").MustBuild()
	d2 := deployment.New().NewID().Workspace(wsID).Version("v2").MustBuild()
	d3 := deployment.New().NewID().Workspace(wsID).Version("v3").MustBuild()
	d4 := deployment.New().NewID().Workspace(wsID2).Version("v1").MustBuild()

	tests := []struct {
		name       string
		init       map[id.DeploymentID]*deployment.Deployment
		wsID       id.WorkspaceID
		pagination *interfaces.PaginationParam
		want       []*deployment.Deployment
		wantInfo   *interfaces.PageBasedInfo
		wantErr    bool
	}{
		{
			name: "page based pagination: first page",
			init: map[id.DeploymentID]*deployment.Deployment{
				d1.ID(): d1,
				d2.ID(): d2,
				d3.ID(): d3,
			},
			wsID: wsID,
			pagination: &interfaces.PaginationParam{
				Page: &interfaces.PageBasedPaginationParam{
					Page:     1,
					PageSize: 2,
				},
			},
			want:     []*deployment.Deployment{d1, d2},
			wantInfo: interfaces.NewPageBasedInfo(3, 1, 2),
		},
		{
			name: "page based pagination: second page",
			init: map[id.DeploymentID]*deployment.Deployment{
				d1.ID(): d1,
				d2.ID(): d2,
				d3.ID(): d3,
			},
			wsID: wsID,
			pagination: &interfaces.PaginationParam{
				Page: &interfaces.PageBasedPaginationParam{
					Page:     2,
					PageSize: 2,
				},
			},
			want:     []*deployment.Deployment{d3},
			wantInfo: interfaces.NewPageBasedInfo(3, 2, 2),
		},
		{
			name: "page based pagination: empty page",
			init: map[id.DeploymentID]*deployment.Deployment{
				d1.ID(): d1,
				d2.ID(): d2,
				d3.ID(): d3,
			},
			wsID: wsID,
			pagination: &interfaces.PaginationParam{
				Page: &interfaces.PageBasedPaginationParam{
					Page:     3,
					PageSize: 2,
				},
			},
			want:     nil,
			wantInfo: interfaces.NewPageBasedInfo(3, 3, 2),
		},
		{
			name: "no pagination",
			init: map[id.DeploymentID]*deployment.Deployment{
				d1.ID(): d1,
				d2.ID(): d2,
				d3.ID(): d3,
			},
			wsID:       wsID,
			pagination: nil,
			want:       []*deployment.Deployment{d1, d2, d3},
			wantInfo:   interfaces.NewPageBasedInfo(3, 1, 3),
		},
		{
			name: "different workspace",
			init: map[id.DeploymentID]*deployment.Deployment{
				d1.ID(): d1,
				d4.ID(): d4,
			},
			wsID:       wsID2,
			pagination: nil,
			want:       []*deployment.Deployment{d4},
			wantInfo:   interfaces.NewPageBasedInfo(1, 1, 1),
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			r := &Deployment{
				data: tt.init,
				f:    repo.WorkspaceFilter{Readable: []id.WorkspaceID{tt.wsID}},
			}

			got, gotInfo, err := r.FindByWorkspace(ctx, tt.wsID, tt.pagination, nil)
			if tt.wantErr {
				assert.Error(t, err)
				return
			}

			assert.NoError(t, err)
			assert.Equal(t, tt.want, got)
			assert.Equal(t, tt.wantInfo, gotInfo)
		})
	}
}
