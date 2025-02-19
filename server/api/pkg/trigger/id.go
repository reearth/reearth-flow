package trigger

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
)

type ID = id.TriggerID
type DeploymentID = id.DeploymentID
type ProjectID = id.ProjectID
type WorkflowID = id.WorkflowID
type WorkspaceID = accountdomain.WorkspaceID

var NewID = id.NewTriggerID

var ErrInvalidID = id.ErrInvalidID
