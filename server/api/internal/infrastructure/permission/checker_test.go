package permission

import (
	"context"
	"errors"
	"testing"

	"github.com/reearth/reearth-accounts/server/pkg/gqlclient/cerbos"
	gqlworkspace "github.com/reearth/reearth-accounts/server/pkg/gqlclient/workspace"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-accounts/server/pkg/workspace"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

type fakeCerbosRepo struct {
	gotParam cerbos.CheckPermissionParam
	result   *cerbos.CheckPermissionResult
	err      error
}

func (f *fakeCerbosRepo) CheckPermission(_ context.Context, param cerbos.CheckPermissionParam) (*cerbos.CheckPermissionResult, error) {
	f.gotParam = param
	return f.result, f.err
}

// embedding the interface satisfies the full method set; we override only FindByID.
type fakeWorkspaceRepo struct {
	gqlworkspace.WorkspaceRepo
	ws  *workspace.Workspace
	err error
}

func (f *fakeWorkspaceRepo) FindByID(_ context.Context, _ string) (*workspace.Workspace, error) {
	return f.ws, f.err
}

func TestChecker_SendsWorkspaceAlias_WhenWorkspaceIDProvided(t *testing.T) {
	cer := &fakeCerbosRepo{result: &cerbos.CheckPermissionResult{Allowed: true}}
	ws := workspace.New().NewID().Alias("acme").MustBuild()
	c := NewChecker(cer, &fakeWorkspaceRepo{ws: ws}, "flow")

	allowed, err := c.CheckPermission(context.Background(), "deployment", "any", ws.ID())

	require.NoError(t, err)
	assert.True(t, allowed)
	require.NotNil(t, cer.gotParam.WorkspaceAlias, "alias must be sent to the Account API")
	assert.Equal(t, "acme", *cer.gotParam.WorkspaceAlias)
}

func TestChecker_NoWorkspaceAlias_WhenNoWorkspaceID(t *testing.T) {
	cer := &fakeCerbosRepo{result: &cerbos.CheckPermissionResult{Allowed: true}}
	c := NewChecker(cer, &fakeWorkspaceRepo{}, "flow")

	_, err := c.CheckPermission(context.Background(), "user", "read")

	require.NoError(t, err)
	assert.Nil(t, cer.gotParam.WorkspaceAlias, "workspace-agnostic checks must not send an alias")
}

func TestChecker_NoWorkspaceAlias_WhenWorkspaceIDNil(t *testing.T) {
	cer := &fakeCerbosRepo{result: &cerbos.CheckPermissionResult{Allowed: true}}
	c := NewChecker(cer, &fakeWorkspaceRepo{}, "flow")

	var zero accountsid.WorkspaceID
	_, err := c.CheckPermission(context.Background(), "deployment", "any", zero)

	require.NoError(t, err)
	assert.Nil(t, cer.gotParam.WorkspaceAlias)
}

func TestChecker_PropagatesWorkspaceLookupError(t *testing.T) {
	cer := &fakeCerbosRepo{result: &cerbos.CheckPermissionResult{Allowed: true}}
	ws := workspace.New().NewID().Alias("acme").MustBuild()
	c := NewChecker(cer, &fakeWorkspaceRepo{err: errors.New("boom")}, "flow")

	allowed, err := c.CheckPermission(context.Background(), "deployment", "any", ws.ID())

	require.Error(t, err, "workspace lookup failure must fail closed (no allow)")
	assert.False(t, allowed)
}
