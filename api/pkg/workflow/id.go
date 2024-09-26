package workflow

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
)

type (
	ID          = id.WorkflowID
	EdgeID      = id.EdgeID
	NodeID      = id.NodeID
	GraphID     = id.GraphID
	ProjectID   = id.ProjectID
	WorkspaceID = accountdomain.WorkspaceID
)

var (
	NewID          = id.NewWorkflowID
	NewNodeID      = id.NewNodeID
	NewEdgeID      = id.NewEdgeID
	NewGraphID     = id.NewGraphID
	NewWorkflowID  = id.NewWorkflowID
	NewProjectID   = id.NewProjectID
	NewWorkspaceID = accountdomain.NewWorkspaceID
)

var (
	MustID          = id.MustWorkflowID
	MustProjectID   = id.MustProjectID
	MustWorkspaceID = id.MustWorkspaceID
)

var (
	IDFrom          = id.WorkflowIDFrom
	ProjectIDFrom   = id.ProjectIDFrom
	WorkspaceIDFrom = accountdomain.WorkspaceIDFrom
)

var (
	IDFromRef          = id.WorkflowIDFromRef
	ProjectIDFromRef   = id.ProjectIDFromRef
	WorkspaceIDFromRef = accountdomain.WorkspaceIDFromRef
)

var ErrInvalidID = id.ErrInvalidID

func MockNewID(pid ID) func() {
	NewID = func() ID { return pid }
	return func() {
		NewID = id.NewWorkflowID
	}
}
