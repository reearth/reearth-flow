package deployment

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
)

type ID = id.DeploymentID
type ProjectID = id.ProjectID
type WorkflowID = id.WorkflowID
type WorkspaceID = accountdomain.WorkspaceID

var NewID = id.NewDeploymentID

var ErrInvalidID = id.ErrInvalidID
