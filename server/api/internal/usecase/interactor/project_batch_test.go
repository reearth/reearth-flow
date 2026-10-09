package interactor

import (
	"context"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestProject_Fetch_NotFoundFirstElementDoesNotPanic pins a crash observed
// in production on the equivalent Deployment.Fetch: FindByIDs pads a
// not-found/unreadable id with nil, and if that nil lands at index 0,
// projects[0].Workspace() panics with a nil pointer dereference. The
// permission check must fall back to the first non-nil element instead.
func TestProject_Fetch_NotFoundFirstElementDoesNotPanic(t *testing.T) {
	ws := accountsid.NewWorkspaceID()
	projectRepo := memory.NewProject()

	p := project.New().NewID().Workspace(ws).Name("p").MustBuild()
	require.NoError(t, projectRepo.Save(context.Background(), p))

	missingID := id.NewProjectID()

	checker := &recordingChecker{allow: true}
	i := &Project{projectRepo: projectRepo, permissionChecker: checker}

	require.NotPanics(t, func() {
		res, err := i.Fetch(context.Background(), []id.ProjectID{missingID, p.ID()})
		require.NoError(t, err)
		require.Len(t, res, 2)
		assert.Nil(t, res[0], "not-found id stays nil in the result")
		require.NotNil(t, res[1])
		require.Len(t, checker.gotWorkspace, 1)
		assert.Equal(t, ws, checker.gotWorkspace[0], "permission check uses the first non-nil element's workspace")
	})
}

// TestProject_Fetch_AllNotFound_UsesNoWorkspacePermissionPath pins the other
// half of the same fix: an all-nil batch must take the no-items permission
// path rather than dereferencing a nil element.
func TestProject_Fetch_AllNotFound_UsesNoWorkspacePermissionPath(t *testing.T) {
	projectRepo := memory.NewProject()
	checker := &recordingChecker{allow: true}
	i := &Project{projectRepo: projectRepo, permissionChecker: checker}

	require.NotPanics(t, func() {
		res, err := i.Fetch(context.Background(), []id.ProjectID{id.NewProjectID(), id.NewProjectID()})
		require.NoError(t, err)
		require.Len(t, res, 2)
		assert.Nil(t, res[0])
		assert.Nil(t, res[1])
		assert.Empty(t, checker.gotWorkspace, "no non-nil element means no workspace-scoped check")
	})
}

// TestProject_Fetch_CrossWorkspaceBatch_DeniedItemOmittedNotLeaked is the
// regression test for the IDOR this fix closes: a batch mixing a project the
// caller can see with one from a workspace they cannot must return the first
// and nil out the second, instead of using the first item's workspace to
// authorize the whole batch.
func TestProject_Fetch_CrossWorkspaceBatch_DeniedItemOmittedNotLeaked(t *testing.T) {
	wsAllowed := accountsid.NewWorkspaceID()
	wsDenied := accountsid.NewWorkspaceID()
	projectRepo := memory.NewProject()

	own := project.New().NewID().Workspace(wsAllowed).Name("own").MustBuild()
	require.NoError(t, projectRepo.Save(context.Background(), own))
	victim := project.New().NewID().Workspace(wsDenied).Name("victim").MustBuild()
	require.NoError(t, projectRepo.Save(context.Background(), victim))

	checker := &multiWorkspaceChecker{allowed: map[accountsid.WorkspaceID]bool{wsAllowed: true, wsDenied: false}}
	i := &Project{projectRepo: projectRepo, permissionChecker: checker}

	res, err := i.Fetch(context.Background(), []id.ProjectID{own.ID(), victim.ID()})
	require.NoError(t, err)
	require.Len(t, res, 2)
	require.NotNil(t, res[0], "the caller's own project must still be returned")
	assert.Equal(t, own.ID(), res[0].ID())
	assert.Nil(t, res[1], "the other tenant's project must not leak just because it rode along with an authorized one")
	assert.Equal(t, 2, checker.calls, "one permission check per distinct workspace in the batch")
}
