package trigger

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type (
	ID           = id.TriggerID
	DeploymentID = id.DeploymentID
	ProjectID    = id.ProjectID
	WorkflowID   = id.WorkflowID
	WorkspaceID  = id.WorkspaceID
)

var NewID = id.NewTriggerID

var ErrInvalidID = id.ErrInvalidID
