package gateway

import (
	"context"
)

type PermissionChecker interface {
	CheckPermission(ctx context.Context, resource string, action string) (bool, error)
}
