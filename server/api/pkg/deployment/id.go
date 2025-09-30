package deployment

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type (
	ID          = id.DeploymentID
	ProjectID   = id.ProjectID
	WorkflowID  = id.WorkflowID
	WorkspaceID = id.WorkspaceID
)

var NewID = id.NewDeploymentID

var ErrInvalidID = id.ErrInvalidID
