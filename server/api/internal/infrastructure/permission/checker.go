package permission

import (
	"context"

	"github.com/reearth/reearth-accounts/server/pkg/gqlclient/cerbos"
	gqlworkspace "github.com/reearth/reearth-accounts/server/pkg/gqlclient/workspace"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
)

type checker struct {
	repo          cerbos.Repo
	workspaceRepo gqlworkspace.WorkspaceRepo
	service       string
}

func NewChecker(repo cerbos.Repo, workspaceRepo gqlworkspace.WorkspaceRepo, service string) gateway.PermissionChecker {
	return &checker{repo: repo, workspaceRepo: workspaceRepo, service: service}
}

func (c *checker) CheckPermission(ctx context.Context, resource string, action string, workspaceID ...accountsid.WorkspaceID) (bool, error) {
	param := cerbos.CheckPermissionParam{
		Service:  c.service,
		Resource: resource,
		Action:   action,
	}

	if len(workspaceID) > 0 {
		wsID := workspaceID[0] // local var → addressable → pointer-receiver IsNil() is safe
		if !wsID.IsNil() {
			alias, err := c.resolveAlias(ctx, wsID)
			if err != nil {
				return false, err // fail closed
			}
			param.WorkspaceAlias = &alias
		}
	}

	result, err := c.repo.CheckPermission(ctx, param)
	if err != nil {
		return false, err
	}
	return result.Allowed, nil
}

// resolveAlias returns the workspace alias for wsID. The accounts client returns a
// non-nil error (never (nil, nil)) when the workspace is missing or unauthorized, so
// any failure propagates and the caller fails closed.
func (c *checker) resolveAlias(ctx context.Context, wsID accountsid.WorkspaceID) (string, error) {
	ws, err := c.workspaceRepo.FindByID(ctx, wsID.String())
	if err != nil {
		return "", err
	}
	return ws.Alias(), nil
}
