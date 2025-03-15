package trigger

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
)

type (
	ID           = id.TriggerID
	DeploymentID = id.DeploymentID
	ProjectID    = id.ProjectID
	WorkflowID   = id.WorkflowID
	WorkspaceID  = accountdomain.WorkspaceID
)

var NewID = id.NewTriggerID

var ErrInvalidID = id.ErrInvalidID
