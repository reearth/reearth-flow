package interactor

import (
	"context"
	"sync"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// countingProjectRepo wraps a real repo.Project and counts calls to FindByIDs,
// so tests can assert a batch of N projects triggers exactly one lookup call.
type countingProjectRepo struct {
	repo.Project
	mu            sync.Mutex
	findByIDsHits int
}

func (r *countingProjectRepo) FindByIDs(ctx context.Context, ids id.ProjectIDList) ([]*project.Project, error) {
	r.mu.Lock()
	r.findByIDsHits++
	r.mu.Unlock()
	return r.Project.FindByIDs(ctx, ids)
}

// multiWorkspaceChecker allows/denies per workspace and counts every call, so
// tests can assert permission checks are consolidated per distinct workspace
// rather than repeated once per item.
type multiWorkspaceChecker struct {
	allowed map[accountsid.WorkspaceID]bool
	mu      sync.Mutex
	calls   int
}

func (c *multiWorkspaceChecker) CheckPermission(_ context.Context, _, _ string, workspaceID ...accountsid.WorkspaceID) (bool, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.calls++
	if len(workspaceID) == 0 {
		return false, nil
	}
	return c.allowed[workspaceID[0]], nil
}

func newProjectAndDeployment(t *testing.T, projectRepo repo.Project, deploymentRepo repo.Deployment, ws accountsid.WorkspaceID) id.ProjectID {
	t.Helper()
	p := project.New().NewID().Workspace(ws).Name("p").MustBuild()
	require.NoError(t, projectRepo.Save(context.Background(), p))

	pid := p.ID()
	d := deployment.New().NewID().Workspace(ws).Project(&pid).IsHead(true).Version("v1").MustBuild()
	require.NoError(t, deploymentRepo.Save(context.Background(), d))

	return pid
}

func TestDeployment_FindByProjects_OneRepoCallOnePermissionCallPerWorkspace(t *testing.T) {
	ws := accountsid.NewWorkspaceID()
	projectRepo := &countingProjectRepo{Project: memory.NewProject()}
	deploymentRepo := memory.NewDeployment()

	var pids []id.ProjectID
	for i := 0; i < 3; i++ {
		pids = append(pids, newProjectAndDeployment(t, projectRepo, deploymentRepo, ws))
	}

	checker := &multiWorkspaceChecker{allowed: map[accountsid.WorkspaceID]bool{ws: true}}
	i := &Deployment{projectRepo: projectRepo, deploymentRepo: deploymentRepo, permissionChecker: checker}

	res, err := i.FindByProjects(context.Background(), pids)
	require.NoError(t, err)

	assert.Equal(t, 1, projectRepo.findByIDsHits, "3 projects should cost one FindByIDs call, not three")
	assert.Equal(t, 1, checker.calls, "one workspace should cost one permission check, not three")
	assert.Len(t, res, 3)
	for _, pid := range pids {
		require.Contains(t, res, pid)
		assert.NotNil(t, res[pid])
	}
}

func TestDeployment_FindByProjects_DeniedWorkspaceOmittedNotErrored(t *testing.T) {
	wsAllowed := accountsid.NewWorkspaceID()
	wsDenied := accountsid.NewWorkspaceID()
	projectRepo := &countingProjectRepo{Project: memory.NewProject()}
	deploymentRepo := memory.NewDeployment()

	allowedPID := newProjectAndDeployment(t, projectRepo, deploymentRepo, wsAllowed)
	deniedPID := newProjectAndDeployment(t, projectRepo, deploymentRepo, wsDenied)

	checker := &multiWorkspaceChecker{allowed: map[accountsid.WorkspaceID]bool{wsAllowed: true, wsDenied: false}}
	i := &Deployment{projectRepo: projectRepo, deploymentRepo: deploymentRepo, permissionChecker: checker}

	res, err := i.FindByProjects(context.Background(), []id.ProjectID{allowedPID, deniedPID})
	require.NoError(t, err)

	assert.Contains(t, res, allowedPID, "caller can see this workspace, its deployment must be returned")
	assert.NotContains(t, res, deniedPID, "caller was denied on this workspace, its deployment must not leak through the batch")
}

// TestDeployment_Fetch_NotFoundFirstElementDoesNotPanic pins a crash observed
// in production: FindByIDs pads a not-found/unreadable id with nil, and if
// that nil lands at index 0, deployments[0].Workspace() panics with a nil
// pointer dereference. The permission check must fall back to the first
// non-nil element instead of assuming index 0 is populated.
func TestDeployment_Fetch_NotFoundFirstElementDoesNotPanic(t *testing.T) {
	ws := accountsid.NewWorkspaceID()
	projectRepo := memory.NewProject()
	deploymentRepo := memory.NewDeployment()
	newProjectAndDeployment(t, projectRepo, deploymentRepo, ws)

	realDeployments, _, err := deploymentRepo.FindByWorkspace(context.Background(), ws, nil, nil)
	require.NoError(t, err)
	require.Len(t, realDeployments, 1)
	realID := realDeployments[0].ID()

	missingID := id.NewDeploymentID()

	checker := &recordingChecker{allow: true}
	i := &Deployment{projectRepo: projectRepo, deploymentRepo: deploymentRepo, permissionChecker: checker}

	require.NotPanics(t, func() {
		res, err := i.Fetch(context.Background(), []id.DeploymentID{missingID, realID})
		require.NoError(t, err)
		require.Len(t, res, 2)
		assert.Nil(t, res[0], "not-found id stays nil in the result")
		require.NotNil(t, res[1])
		require.Len(t, checker.gotWorkspace, 1)
		assert.Equal(t, ws, checker.gotWorkspace[0], "permission check uses the first non-nil element's workspace")
	})
}

// TestDeployment_Fetch_AllNotFound_UsesNoWorkspacePermissionPath pins the
// other half of the same fix: when every id in the batch is unreadable or
// missing, the permission check must take the no-items path rather than
// dereferencing a nil element.
func TestDeployment_Fetch_AllNotFound_UsesNoWorkspacePermissionPath(t *testing.T) {
	projectRepo := memory.NewProject()
	deploymentRepo := memory.NewDeployment()
	checker := &recordingChecker{allow: true}
	i := &Deployment{projectRepo: projectRepo, deploymentRepo: deploymentRepo, permissionChecker: checker}

	require.NotPanics(t, func() {
		res, err := i.Fetch(context.Background(), []id.DeploymentID{id.NewDeploymentID(), id.NewDeploymentID()})
		require.NoError(t, err)
		require.Len(t, res, 2)
		assert.Nil(t, res[0])
		assert.Nil(t, res[1])
		assert.Empty(t, checker.gotWorkspace, "no non-nil element means no workspace-scoped check")
	})
}

func TestDeployment_FindByProjects_EmptyBatch_DeniesAndTouchesNothing(t *testing.T) {
	projectRepo := &countingProjectRepo{Project: memory.NewProject()}
	deploymentRepo := memory.NewDeployment()
	checker := &multiWorkspaceChecker{allowed: map[accountsid.WorkspaceID]bool{}}
	i := &Deployment{projectRepo: projectRepo, deploymentRepo: deploymentRepo, permissionChecker: checker}

	res, err := i.FindByProjects(context.Background(), nil)

	require.Error(t, err, "an empty batch must not silently pass through as an empty allow")
	assert.Nil(t, res)
	assert.Equal(t, 0, projectRepo.findByIDsHits, "no repo call should happen for a denied empty batch")
}
