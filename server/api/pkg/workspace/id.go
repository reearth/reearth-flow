package workspace

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type (
	ID            = id.WorkspaceID
	UserID        = id.UserID
	IntegrationID = id.IntegrationID
)

var NewID = id.NewWorkspaceID

var (
	IDFrom     = id.WorkspaceIDFrom
	UserIDFrom = id.UserIDFrom
)
