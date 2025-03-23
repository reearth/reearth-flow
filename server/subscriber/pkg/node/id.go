package node

import "github.com/reearth/reearthx/idx"

type (
	NodeExecutionID struct{}
)

func (NodeExecutionID) Type() string { return "edgeExecution" }

type (
	ID = idx.ID[NodeExecutionID]
)

var (
	NewID  = idx.New[NodeExecutionID]
	FromID = idx.From[NodeExecutionID]
)
