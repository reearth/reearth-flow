package project

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
)

type (
	ID          = id.ProjectID
	WorkspaceID = accountdomain.WorkspaceID
	WorkflowID  = id.WorkflowID
)

var (
	NewID          = id.NewProjectID
	NewWorkspaceID = accountdomain.NewWorkspaceID
	NewWorkflowID  = id.NewWorkflowID
)

var (
	MustID          = id.MustProjectID
	MustWorkspaceID = id.MustWorkspaceID
)

var (
	IDFrom          = id.ProjectIDFrom
	WorkspaceIDFrom = accountdomain.WorkspaceIDFrom
)

var (
	IDFromRef          = id.ProjectIDFromRef
	WorkspaceIDFromRef = accountdomain.WorkspaceIDFromRef
)

var ErrInvalidID = id.ErrInvalidID

func MockNewID(pid ID) func() {
	NewID = func() ID { return pid }
	return func() {
		NewID = id.NewProjectID
	}
}
