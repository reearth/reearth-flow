package workerconfig

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
)

type ID = id.WorkerConfigID
type WorkspaceID = accountdomain.WorkspaceID

var NewID = id.NewWorkerConfigID
var NewWorkspaceID = accountdomain.NewWorkspaceID

var MustID = id.MustWorkerConfigID
var MustWorkspaceID = accountdomain.MustWorkspaceID

var IDFrom = id.WorkerConfigIDFrom
var IDFromRef = id.WorkerConfigIDFromRef

var WorkspaceIDFrom = accountdomain.WorkspaceIDFrom
var WorkspaceIDFromRef = accountdomain.WorkspaceIDFromRef

var ErrInvalidID = id.ErrInvalidID
