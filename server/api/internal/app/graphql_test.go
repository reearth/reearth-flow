package app

import (
	"context"
	"testing"

	"github.com/99designs/gqlgen/graphql"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestAttachOperationPermissionMemo_ScopedPerOperation pins the fix for
// gqlgen's websocket transport deriving every operation on a connection from
// the ctx of the original upgrade request: two operations sharing the same
// parent (connection) ctx must not share a permission-verdict memo, while
// checks within a single operation must.
func TestAttachOperationPermissionMemo_ScopedPerOperation(t *testing.T) {
	// connCtx simulates the long-lived ctx gqlgen's websocket transport
	// derives every operation from (captured once at connection upgrade).
	connCtx := context.Background()

	// Operation 1: two checks with identical args must hit one memo entry.
	var op1Ctx context.Context
	resp1 := attachOperationPermissionMemo(connCtx, func(ctx context.Context) graphql.ResponseHandler {
		op1Ctx = ctx
		return graphql.OneShot(&graphql.Response{})
	})
	resp1(connCtx)
	require.NotNil(t, op1Ctx)

	memo1 := adapter.PermissionVerdicts(op1Ctx)
	require.NotNil(t, memo1, "operation must have a memo attached")
	_, ok := memo1.Get("user-1", "project", "create", "ws-1")
	assert.False(t, ok, "memo starts empty for a fresh operation")
	memo1.Set("user-1", "project", "create", "ws-1", true)
	allowed, ok := memo1.Get("user-1", "project", "create", "ws-1")
	require.True(t, ok)
	assert.True(t, allowed, "second check within the same operation reuses the first verdict")

	// Operation 2 on the SAME connection (same parent connCtx) — this is what
	// happens for every subsequent operation sent over one websocket
	// connection — must NOT see operation 1's memoized verdict.
	var op2Ctx context.Context
	resp2 := attachOperationPermissionMemo(connCtx, func(ctx context.Context) graphql.ResponseHandler {
		op2Ctx = ctx
		return graphql.OneShot(&graphql.Response{})
	})
	resp2(connCtx)
	require.NotNil(t, op2Ctx)

	memo2 := adapter.PermissionVerdicts(op2Ctx)
	require.NotNil(t, memo2)
	assert.NotSame(t, memo1, memo2, "each operation must get its own memo instance")
	_, ok = memo2.Get("user-1", "project", "create", "ws-1")
	assert.False(t, ok, "a new operation on the same connection must not reuse a stale verdict")
}
