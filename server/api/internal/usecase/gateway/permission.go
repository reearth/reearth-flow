package gateway

import (
	"context"

	"github.com/reearth/reearthx/appx"
)

type PermissionChecker interface {
	CheckPermission(ctx context.Context, authInfo *appx.AuthInfo, resource string, action string) (bool, error)
}
