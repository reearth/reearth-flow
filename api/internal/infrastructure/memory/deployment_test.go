package memory

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/stretchr/testify/assert"
)

func TestDeployment(t *testing.T) {
	ctx := context.Background()
	wsID, _ := accountdomain.WorkspaceIDFrom("workspace1")
	projectID := id.NewProjectID()
	deploymentID := id.NewDeploymentID()
	version := "v1"

	d := deployment.New().
		ID(deploymentID).
		Workspace(wsID).
		Project(&projectID).
		Version(version).
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
		results, pageInfo, err := repo.FindByWorkspace(ctx, wsID, nil)
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
