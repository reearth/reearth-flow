package workflow

import (
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type (
	ID          = id.WorkflowID
	ProjectID   = id.ProjectID
	WorkspaceID = accountsid.WorkspaceID
)

var (
	NewProjectID   = id.NewProjectID
	NewWorkspaceID = accountsid.NewWorkspaceID
)

var (
	MustID          = id.MustWorkflowID
	MustProjectID   = id.MustProjectID
	MustWorkspaceID = accountsid.MustWorkspaceID
)

var (
	IDFrom          = id.WorkflowIDFrom
	ProjectIDFrom   = id.ProjectIDFrom
	WorkspaceIDFrom = accountsid.WorkspaceIDFrom
)

var (
	IDFromRef          = id.WorkflowIDFromRef
	ProjectIDFromRef   = id.ProjectIDFromRef
	WorkspaceIDFromRef = accountsid.WorkspaceIDFromRef
)

var ErrInvalidID = id.ErrInvalidID
