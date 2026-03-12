package job

import (
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type (
	ID           = id.JobID
	DeploymentID = id.DeploymentID
	WorkflowID   = id.WorkflowID
	WorkspaceID  = accountsid.WorkspaceID
)

var NewID = id.NewJobID

var DeploymentIDFrom = id.DeploymentIDFrom

var ErrInvalidID = id.ErrInvalidID
