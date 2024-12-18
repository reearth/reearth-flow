package parameter

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type (
	ID        = id.ParameterID
	ProjectID = id.ProjectID
)

var (
	NewID        = id.NewParameterID
	NewProjectID = id.NewProjectID
)

var (
	MustID        = id.MustParameterID
	MustProjectID = id.MustProjectID
)

var (
	IDFrom        = id.ParameterIDFrom
	ProjectIDFrom = id.ProjectIDFrom
)

var (
	IDFromRef        = id.ParameterIDFromRef
	ProjectIDFromRef = id.ProjectIDFromRef
)

var ErrInvalidID = id.ErrInvalidID

func MockNewID(pid ID) func() {
	NewID = func() ID { return pid }
	return func() {
		NewID = id.NewParameterID
	}
}
