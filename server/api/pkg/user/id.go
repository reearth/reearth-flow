package user

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type (
	ID              = id.UserID
	WorkspaceID     = id.WorkspaceID
	WorkspaceIDList = id.WorkspaceIDList
)

var NewID = id.NewUserID

var (
	IDFrom          = id.UserIDFrom
	WorkspaceIDFrom = id.WorkspaceIDFrom
)
