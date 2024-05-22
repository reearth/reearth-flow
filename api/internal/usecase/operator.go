package usecase

import (
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/reearth/reearthx/account/accountusecase"
)

type Operator struct {
	AcOperator *accountusecase.Operator
}

func (o *Operator) Workspaces(r workspace.Role) accountdomain.WorkspaceIDList {
	if o == nil {
		return nil
	}
	if r == workspace.RoleReader {
		return o.AcOperator.ReadableWorkspaces
	}
	if r == workspace.RoleWriter {
		return o.AcOperator.WritableWorkspaces
	}
	if r == workspace.RoleMaintainer {
		return o.AcOperator.MaintainableWorkspaces
	}
	if r == workspace.RoleOwner {
		return o.AcOperator.OwningWorkspaces
	}
	return nil
}

func (o *Operator) AllReadableWorkspaces() user.WorkspaceIDList {
	return o.AcOperator.AllReadableWorkspaces()
}

func (o *Operator) AllWritableWorkspaces() user.WorkspaceIDList {
	return o.AcOperator.AllWritableWorkspaces()
}

func (o *Operator) AllMaintainingWorkspace() user.WorkspaceIDList {
	return o.AcOperator.AllMaintainingWorkspaces()
}

func (o *Operator) AllOwningWorkspaces() user.WorkspaceIDList {
	return o.AcOperator.AllOwningWorkspaces()
}

func (o *Operator) IsReadableWorkspace(ws ...accountdomain.WorkspaceID) bool {
	return o.AcOperator.IsReadableWorkspace(ws...)
}

func (o *Operator) IsWritableWorkspace(ws ...accountdomain.WorkspaceID) bool {
	return o.AcOperator.IsWritableWorkspace(ws...)
}

func (o *Operator) IsMaintainingWorkspace(ws ...accountdomain.WorkspaceID) bool {
	return o.AcOperator.IsMaintainingWorkspace(ws...)
}

func (o *Operator) IsOwningWorkspace(ws ...accountdomain.WorkspaceID) bool {
	return o.AcOperator.IsOwningWorkspace(ws...)
}

func (o *Operator) AddNewWorkspace(ws accountdomain.WorkspaceID) {
	o.AcOperator.OwningWorkspaces = append(o.AcOperator.OwningWorkspaces, ws)
}
