package interfaces

import (
	"context"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
)

type Workspace interface {
	FindByIDs(context.Context, accountsid.WorkspaceIDList) (accountsworkspace.List, error)
	FindByUser(context.Context, accountsid.UserID) (accountsworkspace.List, error)
	Create(context.Context, string) (*accountsworkspace.Workspace, error)
	Update(context.Context, accountsid.WorkspaceID, string) (*accountsworkspace.Workspace, error)
	Delete(context.Context, accountsid.WorkspaceID) error
	AddUserMember(context.Context, accountsid.WorkspaceID, map[accountsid.UserID]accountsworkspace.Role) (*accountsworkspace.Workspace, error)
	UpdateUserMember(context.Context, accountsid.WorkspaceID, accountsid.UserID, accountsworkspace.Role) (*accountsworkspace.Workspace, error)
	RemoveUserMember(context.Context, accountsid.WorkspaceID, accountsid.UserID) (*accountsworkspace.Workspace, error)
}
