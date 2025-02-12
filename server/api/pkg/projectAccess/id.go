package projectAccess

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type (
	ID        = id.ProjectAccessID
	ProjectID = id.ProjectID
)

var (
	NewID        = id.NewProjectAccessID
	NewProjectID = id.NewProjectID
)

var (
	MustID        = id.MustProjectAccessID
	MustProjectID = id.MustProjectID
)

var (
	IDFrom        = id.ProjectAccessIDFrom
	ProjectIDFrom = id.ProjectIDFrom
)

var (
	IDFromRef        = id.ProjectAccessIDFromRef
	ProjectIDFromRef = id.ProjectIDFromRef
)

var ErrInvalidID = id.ErrInvalidID
