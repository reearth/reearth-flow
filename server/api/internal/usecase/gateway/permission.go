package gateway

import (
	"context"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
)

type PermissionChecker interface {
	CheckPermission(ctx context.Context, resource string, action string, workspaceID ...accountsid.WorkspaceID) (bool, error)
}
