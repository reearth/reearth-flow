package batchconfig

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
)

type ID = id.BatchConfigIDBase
type WorkspaceID = accountdomain.WorkspaceID

var NewID = id.MustBatchConfigID
var NewWorkspaceID = accountdomain.NewWorkspaceID

var MustID = id.MustBatchConfigID
var MustWorkspaceID = accountdomain.MustWorkspaceID

var IDFrom = id.BatchConfigIDFrom
var IDFromRef = id.BatchConfigIDFromRef
var WorkspaceIDFrom = accountdomain.WorkspaceIDFrom

var ErrInvalidID = id.ErrInvalidID
