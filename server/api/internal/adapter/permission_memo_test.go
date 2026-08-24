package adapter

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestPermissionVerdictMemo_SetThenGet(t *testing.T) {
	ctx := AttachPermissionVerdictMemo(context.Background())
	memo := PermissionVerdicts(ctx)

	_, ok := memo.Get("user-1", "project", "create", "ws-1")
	assert.False(t, ok, "unset key must miss")

	memo.Set("user-1", "project", "create", "ws-1", true)
	allowed, ok := memo.Get("user-1", "project", "create", "ws-1")
	assert.True(t, ok)
	assert.True(t, allowed)

	_, ok = memo.Get("user-1", "project", "create", "ws-2")
	assert.False(t, ok, "different workspace must not share the entry")
}

func TestPermissionVerdicts_NotAttached_ReturnsNil(t *testing.T) {
	memo := PermissionVerdicts(context.Background())
	assert.Nil(t, memo)

	// nil receiver must be safe to call.
	_, ok := memo.Get("user-1", "project", "create", "ws-1")
	assert.False(t, ok)
	memo.Set("user-1", "project", "create", "ws-1", true)
}

func TestAttachPermissionVerdictMemo_IsolatedPerContext(t *testing.T) {
	ctx1 := AttachPermissionVerdictMemo(context.Background())
	ctx2 := AttachPermissionVerdictMemo(context.Background())

	PermissionVerdicts(ctx1).Set("u", "r", "a", "w", true)

	_, ok := PermissionVerdicts(ctx2).Get("u", "r", "a", "w")
	assert.False(t, ok, "a fresh memo attachment must not see another request's entries")
}
