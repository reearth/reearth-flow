package gateway

import (
	"context"

	"github.com/reearth/reearthx/appx"
)

type PermissionChecker interface {
	CheckPermission(ctx context.Context, authInfo *appx.AuthInfo, userId string, resource string, action string) (bool, error)
}
