package job

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type (
	ID           = id.JobID
	DeploymentID = id.DeploymentID
	WorkflowID   = id.WorkflowID
	WorkspaceID  = id.WorkspaceID
)

var NewID = id.NewJobID

var DeploymentIDFrom = id.DeploymentIDFrom

var ErrInvalidID = id.ErrInvalidID
