package interactor

import (
	"context"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newProjectAndParameter(t *testing.T, projectRepo repo.Project, paramRepo repo.Parameter, ws accountsid.WorkspaceID) id.ProjectID {
	t.Helper()
	p := project.New().NewID().Workspace(ws).Name("p").MustBuild()
	require.NoError(t, projectRepo.Save(context.Background(), p))

	param, err := parameter.New().ProjectID(p.ID()).Name("param").Type(parameter.TypeText).Build()
	require.NoError(t, err)
	require.NoError(t, paramRepo.Save(context.Background(), param))

	return p.ID()
}

func TestParameter_FetchByProjects_OneRepoCallOnePermissionCallPerWorkspace(t *testing.T) {
	ws := accountsid.NewWorkspaceID()
	projectRepo := &countingProjectRepo{Project: memory.NewProject()}
	paramRepo := memory.NewParameter()

	var pids []id.ProjectID
	for i := 0; i < 3; i++ {
		pids = append(pids, newProjectAndParameter(t, projectRepo, paramRepo, ws))
	}

	checker := &multiWorkspaceChecker{allowed: map[accountsid.WorkspaceID]bool{ws: true}}
	i := &Parameter{projectRepo: projectRepo, paramRepo: paramRepo, permissionChecker: checker}

	res, err := i.FetchByProjects(context.Background(), pids)
	require.NoError(t, err)

	assert.Equal(t, 1, projectRepo.findByIDsHits, "3 projects should cost one FindByIDs call, not three")
	assert.Equal(t, 1, checker.calls, "one workspace should cost one permission check, not three")
	require.Len(t, res, 3)
	for _, pid := range pids {
		require.Contains(t, res, pid)
		require.NotNil(t, res[pid])
		assert.Len(t, *res[pid], 1)
	}
}

func TestParameter_FetchByProjects_DeniedWorkspaceOmittedNotErrored(t *testing.T) {
	wsAllowed := accountsid.NewWorkspaceID()
	wsDenied := accountsid.NewWorkspaceID()
	projectRepo := &countingProjectRepo{Project: memory.NewProject()}
	paramRepo := memory.NewParameter()

	allowedPID := newProjectAndParameter(t, projectRepo, paramRepo, wsAllowed)
	deniedPID := newProjectAndParameter(t, projectRepo, paramRepo, wsDenied)

	checker := &multiWorkspaceChecker{allowed: map[accountsid.WorkspaceID]bool{wsAllowed: true, wsDenied: false}}
	i := &Parameter{projectRepo: projectRepo, paramRepo: paramRepo, permissionChecker: checker}

	res, err := i.FetchByProjects(context.Background(), []id.ProjectID{allowedPID, deniedPID})
	require.NoError(t, err)

	assert.Contains(t, res, allowedPID, "caller can see this workspace, its parameters must be returned")
	assert.NotContains(t, res, deniedPID, "caller was denied on this workspace, its parameters must not leak through the batch")
}

// TestParameter_Fetch_CrossWorkspaceBatch_DeniedItemOmittedNotLeaked is the
// regression test for the IDOR this fix closes: a batch mixing a parameter
// the caller can see with one from a project in a workspace they cannot must
// return the first and omit the second, instead of using the first
// parameter's project to authorize the whole batch.
func TestParameter_Fetch_CrossWorkspaceBatch_DeniedItemOmittedNotLeaked(t *testing.T) {
	wsAllowed := accountsid.NewWorkspaceID()
	wsDenied := accountsid.NewWorkspaceID()
	projectRepo := memory.NewProject()
	paramRepo := memory.NewParameter()

	ownProjectID := newProjectAndParameter(t, projectRepo, paramRepo, wsAllowed)
	victimProjectID := newProjectAndParameter(t, projectRepo, paramRepo, wsDenied)

	ownParams, err := paramRepo.FindByProject(context.Background(), ownProjectID)
	require.NoError(t, err)
	require.Len(t, *ownParams, 1)
	victimParams, err := paramRepo.FindByProject(context.Background(), victimProjectID)
	require.NoError(t, err)
	require.Len(t, *victimParams, 1)

	checker := &multiWorkspaceChecker{allowed: map[accountsid.WorkspaceID]bool{wsAllowed: true, wsDenied: false}}
	i := &Parameter{projectRepo: projectRepo, paramRepo: paramRepo, permissionChecker: checker}

	res, err := i.Fetch(context.Background(), id.ParameterIDList{(*ownParams)[0].ID(), (*victimParams)[0].ID()})
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Len(t, *res, 1, "the other tenant's parameter must not leak just because it rode along with an authorized one")
	assert.Equal(t, (*ownParams)[0].ID(), (*res)[0].ID(), "the caller's own parameter must still be returned")
}

func TestParameter_FetchByProjects_EmptyBatch_DeniesAndTouchesNothing(t *testing.T) {
	projectRepo := &countingProjectRepo{Project: memory.NewProject()}
	paramRepo := memory.NewParameter()
	checker := &multiWorkspaceChecker{allowed: map[accountsid.WorkspaceID]bool{}}
	i := &Parameter{projectRepo: projectRepo, paramRepo: paramRepo, permissionChecker: checker}

	res, err := i.FetchByProjects(context.Background(), nil)

	require.Error(t, err, "an empty batch must not silently pass through as an empty allow")
	assert.Nil(t, res)
	assert.Equal(t, 0, projectRepo.findByIDsHits, "no repo call should happen for a denied empty batch")
}
