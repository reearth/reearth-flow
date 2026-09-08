package adapter

import (
	"context"
	"sync"
)

type permissionVerdictMemoKey struct{}

type permissionVerdictEntry struct {
	user      string
	resource  string
	action    string
	workspace string
}

// PermissionVerdictMemo memoizes permission-check verdicts for the lifetime of
// a single request. It must be attached fresh per request via
// AttachPermissionVerdictMemo — never stored on a long-lived struct, or the
// verdicts would silently become a cross-request, cross-tenant cache.
type PermissionVerdictMemo struct {
	entries map[permissionVerdictEntry]bool
	mu      sync.Mutex
}

func AttachPermissionVerdictMemo(ctx context.Context) context.Context {
	return context.WithValue(ctx, permissionVerdictMemoKey{}, &PermissionVerdictMemo{
		entries: make(map[permissionVerdictEntry]bool),
	})
}

func PermissionVerdicts(ctx context.Context) *PermissionVerdictMemo {
	m, _ := ctx.Value(permissionVerdictMemoKey{}).(*PermissionVerdictMemo)
	return m
}

// Get reports a memoized verdict for the given key, if any. A nil receiver
// (memo not attached to the request) always misses.
func (m *PermissionVerdictMemo) Get(user, resource, action, workspace string) (allowed, ok bool) {
	if m == nil {
		return false, false
	}
	m.mu.Lock()
	defer m.mu.Unlock()
	allowed, ok = m.entries[permissionVerdictEntry{user: user, resource: resource, action: action, workspace: workspace}]
	return allowed, ok
}

// Set stores a verdict — allow or deny, either is a real answer from the
// checker and safe to reuse within the operation. Callers must never call
// this for an error result: the checker failing to reach a verdict is not
// itself a verdict, and the next attempt might succeed.
func (m *PermissionVerdictMemo) Set(user, resource, action, workspace string, allowed bool) {
	if m == nil {
		return
	}
	m.mu.Lock()
	defer m.mu.Unlock()
	m.entries[permissionVerdictEntry{user: user, resource: resource, action: action, workspace: workspace}] = allowed
}
