package interactor

import (
	"context"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// recordingChecker captures the arguments passed to CheckPermission so tests can
// assert that interactors forward the target workspace. `allow` controls whether
// the call is permitted, letting a test short-circuit before later side effects.
type recordingChecker struct {
	gotWorkspace []accountsid.WorkspaceID
	gotResource  string
	gotAction    string
	allow        bool
}

func (r *recordingChecker) CheckPermission(_ context.Context, resource, action string, workspaceID ...accountsid.WorkspaceID) (bool, error) {
	r.gotResource, r.gotAction, r.gotWorkspace = resource, action, workspaceID
	return r.allow, nil
}

func TestDeployment_FindByWorkspace_PassesWorkspaceID(t *testing.T) {
	rc := &recordingChecker{allow: true}
	// nilDeploymentRepo (deployment_test.go) returns (nil,nil,nil) from FindByWorkspace,
	// so the post-check repo call is safe.
	i := &Deployment{permissionChecker: rc, deploymentRepo: &nilDeploymentRepo{}}
	wsID := accountsid.NewWorkspaceID()

	_, _, err := i.FindByWorkspace(context.Background(), wsID, nil, nil)

	require.NoError(t, err)
	assert.Equal(t, rbac.ResourceDeployment, rc.gotResource)
	require.Len(t, rc.gotWorkspace, 1)
	assert.Equal(t, wsID, rc.gotWorkspace[0])
}

func TestDeployment_Create_PassesWorkspaceID(t *testing.T) {
	// allow=false makes Create short-circuit at the permission check (before it
	// touches the transaction/repos), while still recording the workspace argument.
	rc := &recordingChecker{allow: false}
	i := &Deployment{permissionChecker: rc}
	wsID := accountsid.NewWorkspaceID()

	_, err := i.Create(context.Background(), interfaces.CreateDeploymentParam{Workspace: wsID})

	require.ErrorIs(t, err, interfaces.ErrOperationDenied)
	assert.Equal(t, rbac.ResourceDeployment, rc.gotResource)
	require.Len(t, rc.gotWorkspace, 1)
	assert.Equal(t, wsID, rc.gotWorkspace[0])
}
