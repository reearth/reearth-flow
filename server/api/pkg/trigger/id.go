package trigger

import (
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type (
	ID           = id.TriggerID
	DeploymentID = id.DeploymentID
	ProjectID    = id.ProjectID
	WorkflowID   = id.WorkflowID
	WorkspaceID  = accountsid.WorkspaceID
)

var NewID = id.NewTriggerID

var ErrInvalidID = id.ErrInvalidID
