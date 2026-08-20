package interactor

import (
	"context"
	"errors"
	"sync/atomic"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
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

// TestCheckPermission_MemoDoesNotCrossRequests pins the security property that
// makes the memo request-scoped: a role revocation must take effect on the very
// next request. If the memo ever moved onto a long-lived struct, this test
// would fail because the second request would observe the first request's
// (now-stale) verdict instead of asking the checker again.
func TestCheckPermission_MemoDoesNotCrossRequests(t *testing.T) {
	setSkipPermissionCheck(false)
	checker := &countingChecker{allow: true}
	u := accountsuser.New().NewID().Name("hoge").Email("abc@bb.cc").MustBuild()
	wsID := accountsid.NewWorkspaceID()

	req1 := adapter.AttachPermissionVerdictMemo(adapter.AttachUser(context.Background(), u))
	require.NoError(t, checkPermission(req1, checker, "project", "create", wsID))
	require.NoError(t, checkPermission(req1, checker, "project", "create", wsID))
	assert.Equal(t, int32(1), atomic.LoadInt32(&checker.calls), "second identical check within request 1 must be memoized")

	// same user, same resource/action/workspace, but a FRESH request context.
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
