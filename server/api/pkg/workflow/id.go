package workflow

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
)

type (
	ID          = id.WorkflowID
	ProjectID   = id.ProjectID
	WorkspaceID = accountdomain.WorkspaceID
	GraphID     = id.GraphID
)

var (
	NewProjectID   = id.NewProjectID
	NewWorkspaceID = accountdomain.NewWorkspaceID
	NewGraphID     = id.NewGraphID
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
