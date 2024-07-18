package project

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
)

type ID = id.ProjectID
type WorkspaceID = accountdomain.WorkspaceID
type WorkflowID = id.WorkflowID

var NewID = id.NewProjectID
var NewWorkspaceID = accountdomain.NewWorkspaceID
var NewWorkflowID = id.NewWorkflowID

var MustID = id.MustProjectID
var MustWorkspaceID = id.MustWorkspaceID

var IDFrom = id.ProjectIDFrom
var WorkspaceIDFrom = accountdomain.WorkspaceIDFrom

var IDFromRef = id.ProjectIDFromRef
var WorkspaceIDFromRef = accountdomain.WorkspaceIDFromRef

var ErrInvalidID = id.ErrInvalidID

func MockNewID(pid ID) func() {
	NewID = func() ID { return pid }
	return func() {
		NewID = id.NewProjectID
	}
}
