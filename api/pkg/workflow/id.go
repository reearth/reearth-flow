package workflow

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
)

type ID = id.WorkflowID
type ProjectID = id.ProjectID
type WorkspaceID = accountdomain.WorkspaceID

var NewID = id.NewWorkflowID
var NewProjectID = id.NewProjectID
var NewWorkspaceID = accountdomain.NewWorkspaceID

var MustID = id.MustWorkflowID
var MustProjectID = id.MustProjectID
var MustWorkspaceID = id.MustWorkspaceID

var IDFrom = id.WorkflowIDFrom
var ProjectIDFrom = id.ProjectIDFrom
var WorkspaceIDFrom = accountdomain.WorkspaceIDFrom

var IDFromRef = id.WorkflowIDFromRef
var ProjectIDFromRef = id.ProjectIDFromRef
var WorkspaceIDFromRef = accountdomain.WorkspaceIDFromRef

var ErrInvalidID = id.ErrInvalidID

func MockNewID(pid ID) func() {
	NewID = func() ID { return pid }
	return func() {
		NewID = id.NewWorkflowID
	}
}
