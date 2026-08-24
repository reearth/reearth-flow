package interactor

import (
	"context"
	"testing"

	gqlworkspace "github.com/reearth/reearth-accounts/server/pkg/gqlclient/workspace"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-accounts/server/pkg/role"
	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// countingWorkspaceGQLRepo counts writes so tests can assert a denied call
// never reaches the accounts service. Embedding satisfies the full interface.
type countingWorkspaceGQLRepo struct {
	gqlworkspace.WorkspaceRepo
	writes int
}

func (r *countingWorkspaceGQLRepo) UpdateWorkspace(context.Context, gqlworkspace.UpdateWorkspaceInput) (*accountsworkspace.Workspace, error) {
	r.writes++
	return nil, nil
}
func (r *countingWorkspaceGQLRepo) CreateWorkspace(context.Context, gqlworkspace.CreateWorkspaceInput) (*accountsworkspace.Workspace, error) {
	r.writes++
	return nil, nil
}
func (r *countingWorkspaceGQLRepo) DeleteWorkspace(context.Context, string) error {
	r.writes++
	return nil
}
func (r *countingWorkspaceGQLRepo) AddUsersToWorkspace(context.Context, gqlworkspace.AddUsersToWorkspaceInput) (*accountsworkspace.Workspace, error) {
	r.writes++
	return nil, nil
}
func (r *countingWorkspaceGQLRepo) UpdateUserOfWorkspace(context.Context, gqlworkspace.UpdateUserOfWorkspaceInput) (*accountsworkspace.Workspace, error) {
	r.writes++
	return nil, nil
}
func (r *countingWorkspaceGQLRepo) RemoveUserFromWorkspace(context.Context, string, string) (*accountsworkspace.Workspace, error) {
	r.writes++
	return nil, nil
}

func wsMutations() map[string]func(context.Context, interfaces.Workspace, accountsid.WorkspaceID) error {
	uid := accountsid.NewUserID()
	return map[string]func(context.Context, interfaces.Workspace, accountsid.WorkspaceID) error{
		"Update": func(ctx context.Context, i interfaces.Workspace, w accountsid.WorkspaceID) error {
			_, err := i.Update(ctx, w, "n")
			return err
		},
		"Delete": func(ctx context.Context, i interfaces.Workspace, w accountsid.WorkspaceID) error {
			return i.Delete(ctx, w)
		},
		"AddUserMember": func(ctx context.Context, i interfaces.Workspace, w accountsid.WorkspaceID) error {
			_, err := i.AddUserMember(ctx, w, map[accountsid.UserID]role.RoleType{uid: role.RoleReader})
			return err
		},
		"UpdateUserMember": func(ctx context.Context, i interfaces.Workspace, w accountsid.WorkspaceID) error {
			_, err := i.UpdateUserMember(ctx, w, uid, role.RoleOwner)
			return err
		},
		"RemoveUserMember": func(ctx context.Context, i interfaces.Workspace, w accountsid.WorkspaceID) error {
			_, err := i.RemoveUserMember(ctx, w, uid)
			return err
		},
	}
}

// TestWorkspace_MutationsAreAuthorized is the regression this closes: workspace
// membership changes reached the accounts service with no permission check at
// all, so a workspace reader could promote themselves or anyone else. The call
// count is the assertion that matters — an error that still wrote has denied
// nothing.
func TestWorkspace_MutationsAreAuthorized(t *testing.T) {
	for name, call := range wsMutations() {
		t.Run(name, func(t *testing.T) {
			repo := &countingWorkspaceGQLRepo{}
			rc := &recordingChecker{allow: false}
			i := NewWorkspace(repo, rc)

			wid := accountsid.NewWorkspaceID()
			err := call(context.Background(), i, wid)

			assert.ErrorIs(t, err, interfaces.ErrOperationDenied)
			assert.Zero(t, repo.writes, "denied %s still reached the accounts service", name)
			assert.Equal(t, rbac.ResourceWorkspace, rc.gotResource)
			require.Len(t, rc.gotWorkspace, 1, "%s must scope the check to the target workspace", name)
			assert.Equal(t, wid, rc.gotWorkspace[0])
		})
	}
}

func TestWorkspace_MutationsProceedWhenAllowed(t *testing.T) {
	for name, call := range wsMutations() {
		t.Run(name, func(t *testing.T) {
			repo := &countingWorkspaceGQLRepo{}
			i := NewWorkspace(repo, &recordingChecker{allow: true})

			require.NoError(t, call(context.Background(), i, accountsid.NewWorkspaceID()))
			assert.Equal(t, 1, repo.writes, "%s should reach the accounts service once", name)
		})
	}
}

// TestWorkspace_CreateIsNotWorkspaceScoped: creating a workspace has no
// workspace to authorize against, so it must not send one — otherwise the
// check would be evaluated against a workspace that does not exist yet.
func TestWorkspace_CreateIsNotWorkspaceScoped(t *testing.T) {
	rc := &recordingChecker{allow: true}
	i := NewWorkspace(&countingWorkspaceGQLRepo{}, rc)

	_, _ = i.Create(context.Background(), "n")

	assert.Empty(t, rc.gotWorkspace)
}
