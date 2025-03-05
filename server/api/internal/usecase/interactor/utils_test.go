package interactor

import (
	"context"

	"github.com/reearth/reearthx/appx"
)

type mockPermissionChecker struct {
	checkPermissionFunc func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error)
}

func NewMockPermissionChecker(checkFunc func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error)) *mockPermissionChecker {
	return &mockPermissionChecker{
		checkPermissionFunc: checkFunc,
	}
}

func (m *mockPermissionChecker) CheckPermission(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
	if m.checkPermissionFunc != nil {
		return m.checkPermissionFunc(ctx, authInfo, userId, resource, action)
	}
	return true, nil
}
