package workflow

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type (
	ID          = id.WorkflowID
	ProjectID   = id.ProjectID
	WorkspaceID = id.WorkspaceID
)

var (
	NewProjectID   = id.NewProjectID
	NewWorkspaceID = id.NewWorkspaceID
)

var (
	MustID          = id.MustWorkflowID
	MustProjectID   = id.MustProjectID
	MustWorkspaceID = id.MustWorkspaceID
)

var (
	IDFrom          = id.WorkflowIDFrom
	ProjectIDFrom   = id.ProjectIDFrom
	WorkspaceIDFrom = id.WorkspaceIDFrom
)

var (
	IDFromRef          = id.WorkflowIDFromRef
	ProjectIDFromRef   = id.ProjectIDFromRef
	WorkspaceIDFromRef = id.WorkspaceIDFromRef
)

var ErrInvalidID = id.ErrInvalidID
