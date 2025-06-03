package job

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
)

type (
	ID           = id.JobID
	DeploymentID = id.DeploymentID
	WorkflowID   = id.WorkflowID
	WorkspaceID  = accountdomain.WorkspaceID
)

var NewID = id.NewJobID

var DeploymentIDFrom = id.DeploymentIDFrom

var ErrInvalidID = id.ErrInvalidID
