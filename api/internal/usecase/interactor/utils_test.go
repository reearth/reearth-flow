package interactor

import (
	"context"
)

type mockPermissionChecker struct {
	checkPermissionFunc func(ctx context.Context, resource, action string) (bool, error)
}

func NewMockPermissionChecker(checkFunc func(ctx context.Context, resource, action string) (bool, error)) *mockPermissionChecker {
	return &mockPermissionChecker{
		checkPermissionFunc: checkFunc,
	}
}

func (m *mockPermissionChecker) CheckPermission(ctx context.Context, resource, action string) (bool, error) {
	if m.checkPermissionFunc != nil {
		return m.checkPermissionFunc(ctx, resource, action)
	}
	return true, nil
}
