package auth

import (
	"context"
	"errors"

	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/rerror"
)

// UserFinder resolves the user whose JWT is in the context.
type UserFinder interface {
	FindMe(ctx context.Context) (*accountsuser.User, error)
}

// ProjectDocumentAccess checks websocket access to a project's live document.
//
// It uses the project resource, as the document API does (interactor.Websocket):
// Cerbos denies resources absent from its policy store, and projectDocument is
// not published there. project:read admits readers; project:any (writer and
// above) grants editing, since project:edit is maintainer-only.
type ProjectDocumentAccess struct {
	Projects repo.Project
	Access   repo.ProjectAccess
	Users    UserFinder
	Checker  gateway.PermissionChecker
	// SkipPermissionCheck mirrors the API's development switch: the user and the
	// project must still exist, but the policy check is skipped.
	SkipPermissionCheck bool
}

var _ ProjectAccessChecker = (*ProjectDocumentAccess)(nil)

func (a *ProjectDocumentAccess) UserAccess(ctx context.Context, rawProjectID string) (bool, bool, error) {
	pid, err := id.ProjectIDFrom(rawProjectID)
	if err != nil {
		return false, false, nil
	}
	u, err := a.Users.FindMe(ctx)
	if err != nil {
		return false, false, err
	}
	if u == nil {
		return false, false, nil
	}
	ctx = adapter.AttachUser(ctx, u)

	p, err := a.Projects.FindByID(ctx, pid)
	if errors.Is(err, rerror.ErrNotFound) || (err == nil && p == nil) {
		return false, false, nil
	}
	if err != nil {
		return false, false, err
	}
	if a.SkipPermissionCheck {
		return true, true, nil
	}

	read, err := a.Checker.CheckPermission(ctx, rbac.ResourceProject, rbac.ActionRead, p.Workspace())
	if err != nil || !read {
		return false, false, err
	}
	edit, err := a.Checker.CheckPermission(ctx, rbac.ResourceProject, rbac.ActionAny, p.Workspace())
	if err != nil {
		return false, false, err
	}
	return true, edit, nil
}

func (a *ProjectDocumentAccess) SharedAccess(ctx context.Context, shareToken, rawProjectID string) (bool, error) {
	pid, err := id.ProjectIDFrom(rawProjectID)
	if err != nil {
		return false, nil
	}
	pa, err := a.Access.FindByToken(ctx, shareToken)
	if errors.Is(err, rerror.ErrNotFound) || (err == nil && pa == nil) {
		return false, nil
	}
	if err != nil {
		return false, err
	}
	return pa.IsPublic() && pa.Project() == pid, nil
}
