package project

import (
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type (
	ID          = id.ProjectID
	WorkspaceID = accountsid.WorkspaceID
	WorkflowID  = id.WorkflowID
)

var (
	NewID          = id.NewProjectID
	NewWorkspaceID = accountsid.NewWorkspaceID
	NewWorkflowID  = id.NewWorkflowID
)

var (
	MustID          = id.MustProjectID
	MustWorkspaceID = accountsid.MustWorkspaceID
)

var (
	IDFrom          = id.ProjectIDFrom
	WorkspaceIDFrom = accountsid.WorkspaceIDFrom
)

var (
	IDFromRef          = id.ProjectIDFromRef
	WorkspaceIDFromRef = accountsid.WorkspaceIDFromRef
)

var ErrInvalidID = id.ErrInvalidID

func MockNewID(pid ID) func() {
	NewID = func() ID { return pid }
	return func() {
		NewID = id.NewProjectID
	}
}
