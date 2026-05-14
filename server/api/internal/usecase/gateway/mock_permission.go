package gateway

import "context"

type MockPermissionChecker struct {
	Error error
	Allow bool
}

var _ PermissionChecker = (*MockPermissionChecker)(nil)

func NewMockPermissionChecker() *MockPermissionChecker {
	return &MockPermissionChecker{
		Allow: true,
		Error: nil,
	}
}

func (m *MockPermissionChecker) CheckPermission(_ context.Context, _, _ string) (bool, error) {
	if m.Error != nil {
		return false, m.Error
	}
	return m.Allow, nil
}
