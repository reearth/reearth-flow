package project

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type (
	ID          = id.ProjectID
	WorkspaceID = id.WorkspaceID
	WorkflowID  = id.WorkflowID
)

var (
	NewID          = id.NewProjectID
	NewWorkspaceID = id.NewWorkspaceID
	NewWorkflowID  = id.NewWorkflowID
)

var (
	MustID          = id.MustProjectID
	MustWorkspaceID = id.MustWorkspaceID
)

var (
	IDFrom          = id.ProjectIDFrom
	WorkspaceIDFrom = id.WorkspaceIDFrom
)

var (
	IDFromRef          = id.ProjectIDFromRef
	WorkspaceIDFromRef = id.WorkspaceIDFromRef
)

var ErrInvalidID = id.ErrInvalidID

func MockNewID(pid ID) func() {
	NewID = func() ID { return pid }
	return func() {
		NewID = id.NewProjectID
	}
}
