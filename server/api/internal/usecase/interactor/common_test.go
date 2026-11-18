package interactor

import (
	"context"
	"errors"
	"sync/atomic"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestCheckPermission(t *testing.T) {
	checkerAllow := NewMockPermissionChecker(func(_ context.Context, _, _ string) (bool, error) {
		return true, nil
	})
	checkerDeny := NewMockPermissionChecker(func(_ context.Context, _, _ string) (bool, error) {
		return false, nil
	})
	checkerErr := NewMockPermissionChecker(func(_ context.Context, _, _ string) (bool, error) {
		return false, errors.New("service unavailable")
	})

	tests := []struct {
		ctx     context.Context
		checker *mockPermissionChecker
		wantErr error
		name    string
	}{
		{
			name:    "grants permission when checker allows",
			ctx:     context.Background(),
			checker: checkerAllow,
			wantErr: nil,
		},
		{
			name:    "denies when checker returns false",
			ctx:     context.Background(),
			checker: checkerDeny,
			wantErr: interfaces.ErrOperationDenied,
		},
		{
			name:    "propagates error from checker",
			ctx:     context.Background(),
			checker: checkerErr,
			wantErr: errors.New("service unavailable"),
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			setSkipPermissionCheck(false)
			err := checkPermission(tt.ctx, tt.checker, "project", "create")
			if tt.wantErr == nil {
				assert.NoError(t, err)
			} else {
				assert.EqualError(t, err, tt.wantErr.Error())
			}
		})
	}

	t.Run("skips check entirely when skipPermissionCheck is true", func(t *testing.T) {
		setSkipPermissionCheck(true)
		defer setSkipPermissionCheck(false)
		err := checkPermission(context.Background(), checkerDeny, "project", "delete")
		assert.NoError(t, err)
	})
}

// countingChecker records how many times CheckPermission was actually invoked.
type countingChecker struct {
	err   error
	calls int32
	allow bool
}

func (c *countingChecker) CheckPermission(_ context.Context, _, _ string, _ ...accountsid.WorkspaceID) (bool, error) {
	atomic.AddInt32(&c.calls, 1)
	return c.allow, c.err
}

func newRequestCtxWithUser(t *testing.T) context.Context {
	t.Helper()
	u := accountsuser.New().NewID().Name("hoge").Email("abc@bb.cc").MustBuild()
	ctx := adapter.AttachUser(context.Background(), u)
	return adapter.AttachPermissionVerdictMemo(ctx)
}

func TestCheckPermission_MemoWithinSameRequest_CallsCheckerOnce(t *testing.T) {
	setSkipPermissionCheck(false)
	checker := &countingChecker{allow: true}
	ctx := newRequestCtxWithUser(t)
	wsID := accountsid.NewWorkspaceID()

	require.NoError(t, checkPermission(ctx, checker, "project", "create", wsID))
	require.NoError(t, checkPermission(ctx, checker, "project", "create", wsID))

	assert.Equal(t, int32(1), atomic.LoadInt32(&checker.calls), "identical checks within one request must hit the checker only once")
}

func TestCheckPermission_MemoDeniedVerdict_AlsoMemoized(t *testing.T) {
	setSkipPermissionCheck(false)
	checker := &countingChecker{allow: false}
	ctx := newRequestCtxWithUser(t)
	wsID := accountsid.NewWorkspaceID()

	err1 := checkPermission(ctx, checker, "project", "delete", wsID)
	err2 := checkPermission(ctx, checker, "project", "delete", wsID)

	require.ErrorIs(t, err1, interfaces.ErrOperationDenied)
	require.ErrorIs(t, err2, interfaces.ErrOperationDenied)
	assert.Equal(t, int32(1), atomic.LoadInt32(&checker.calls))
}

// TestCheckPermission_MemoDoesNotCrossRequests: a role revocation must take
// effect on the very next request, so the memo must stay request-scoped.
func TestCheckPermission_MemoDoesNotCrossRequests(t *testing.T) {
	setSkipPermissionCheck(false)
	checker := &countingChecker{allow: true}
	u := accountsuser.New().NewID().Name("hoge").Email("abc@bb.cc").MustBuild()
	wsID := accountsid.NewWorkspaceID()

	req1 := adapter.AttachPermissionVerdictMemo(adapter.AttachUser(context.Background(), u))
	require.NoError(t, checkPermission(req1, checker, "project", "create", wsID))
	require.NoError(t, checkPermission(req1, checker, "project", "create", wsID))
	assert.Equal(t, int32(1), atomic.LoadInt32(&checker.calls), "second identical check within request 1 must be memoized")

	req2 := adapter.AttachPermissionVerdictMemo(adapter.AttachUser(context.Background(), u))
	require.NoError(t, checkPermission(req2, checker, "project", "create", wsID))
	assert.Equal(t, int32(2), atomic.LoadInt32(&checker.calls), "a new request must re-ask the checker, not reuse request 1's verdict")
}

func TestCheckPermission_MemoErrorNotCached(t *testing.T) {
	setSkipPermissionCheck(false)
	checker := &countingChecker{err: errors.New("service unavailable")}
	ctx := newRequestCtxWithUser(t)
	wsID := accountsid.NewWorkspaceID()

	err1 := checkPermission(ctx, checker, "project", "create", wsID)
	err2 := checkPermission(ctx, checker, "project", "create", wsID)

	require.Error(t, err1)
	require.Error(t, err2)
	assert.Equal(t, int32(2), atomic.LoadInt32(&checker.calls), "a checker error must never be memoized as a verdict")
}

// gqlgen derives every operation on a websocket connection from the upgrade
// request's ctx. AroundOperations in internal/app/graphql.go therefore attaches
// a fresh memo per operation, so a long-lived connection cannot reuse a stale
// verdict.
func TestCheckPermission_MemoIsPerOperation_NotPerWebsocketConnection(t *testing.T) {
	setSkipPermissionCheck(false)
	checker := &countingChecker{allow: true}
	u := accountsuser.New().NewID().Name("hoge").Email("abc@bb.cc").MustBuild()
	wsID := accountsid.NewWorkspaceID()

	// connCtx stands in for the ctx captured once at websocket upgrade, which
	// has no memo of its own.
	connCtx := adapter.AttachUser(context.Background(), u)

	op1Ctx := adapter.AttachPermissionVerdictMemo(connCtx)
	require.NoError(t, checkPermission(op1Ctx, checker, "project", "create", wsID))
	require.NoError(t, checkPermission(op1Ctx, checker, "project", "create", wsID))
	assert.Equal(t, int32(1), atomic.LoadInt32(&checker.calls), "two checks within one operation must hit the checker once")

	op2Ctx := adapter.AttachPermissionVerdictMemo(connCtx)
	require.NoError(t, checkPermission(op2Ctx, checker, "project", "create", wsID))
	assert.Equal(t, int32(2), atomic.LoadInt32(&checker.calls), "a second operation on the same websocket connection must not reuse operation 1's cached verdict")
}

func TestCheckPermission_MemoDifferentiatesByKey(t *testing.T) {
	setSkipPermissionCheck(false)
	checker := &countingChecker{allow: true}
	ctx := newRequestCtxWithUser(t)
	wsID1 := accountsid.NewWorkspaceID()
	wsID2 := accountsid.NewWorkspaceID()

	require.NoError(t, checkPermission(ctx, checker, "project", "create", wsID1))
	require.NoError(t, checkPermission(ctx, checker, "project", "create", wsID2))
	require.NoError(t, checkPermission(ctx, checker, "deployment", "create", wsID1))
	require.NoError(t, checkPermission(ctx, checker, "project", "edit", wsID1))

	assert.Equal(t, int32(4), atomic.LoadInt32(&checker.calls), "distinct resource/action/workspace combinations must not share a memo entry")
}

// pgxx.OrderByIDs, used by the Postgres repos, drops not-found ids and compacts
// the slice instead of nil-padding it. The result must still be re-aligned to
// the requested ids.
func TestAlignToRequestedIDs_CompactedResultIsReexpanded(t *testing.T) {
	type item struct{ id int }
	ids := []int{1, 2, 3, 4}
	// Simulate pgxx.OrderByIDs: id 2 and 4 are missing, and the backend
	// returned the found rows out of request order.
	compacted := []*item{{id: 3}, {id: 1}}

	got := alignToRequestedIDs(ids, compacted, nil, func(it *item) int { return it.id })

	require.Len(t, got, len(ids))
	require.NotNil(t, got[0])
	assert.Equal(t, 1, got[0].id)
	assert.Nil(t, got[1], "id 2 was dropped by the backend and must be nil, not shift later items into its slot")
	require.NotNil(t, got[2])
	assert.Equal(t, 3, got[2].id)
	assert.Nil(t, got[3], "id 4 was dropped by the backend and must be nil")
}

// compactingProjectRepo drops not-found ids instead of nil-padding them,
// mirroring pgxx.OrderByIDs.
type compactingProjectRepo struct {
	repo.Project
}

func (r *compactingProjectRepo) FindByIDs(ctx context.Context, ids id.ProjectIDList) ([]*project.Project, error) {
	found, err := r.Project.FindByIDs(ctx, ids)
	if err != nil {
		return nil, err
	}
	compacted := make([]*project.Project, 0, len(found))
	for _, p := range found {
		if p != nil {
			compacted = append(compacted, p)
		}
	}
	return compacted, nil
}

func TestProject_Fetch_PostgresStyleCompactedRepo_StillAuthorizesAndAligns(t *testing.T) {
	wsAllowed := accountsid.NewWorkspaceID()
	wsDenied := accountsid.NewWorkspaceID()
	projectRepo := &compactingProjectRepo{Project: memory.NewProject()}

	own := project.New().NewID().Workspace(wsAllowed).Name("own").MustBuild()
	require.NoError(t, projectRepo.Save(context.Background(), own))
	victim := project.New().NewID().Workspace(wsDenied).Name("victim").MustBuild()
	require.NoError(t, projectRepo.Save(context.Background(), victim))
	missingID := id.NewProjectID()

	checker := &multiWorkspaceChecker{allowed: map[accountsid.WorkspaceID]bool{wsAllowed: true, wsDenied: false}}
	i := &Project{projectRepo: projectRepo, permissionChecker: checker}

	res, err := i.Fetch(context.Background(), []id.ProjectID{missingID, own.ID(), victim.ID()})
	require.NoError(t, err)
	require.Len(t, res, 3, "result must stay aligned to the request even though the repo compacted its own response")
	assert.Nil(t, res[0], "the missing id must land as nil at its own requested position")
	require.NotNil(t, res[1], "the caller's own project must be returned at its own requested position")
	assert.Equal(t, own.ID(), res[1].ID())
	assert.Nil(t, res[2], "the other tenant's project must not leak")
}
