package id

import "github.com/reearth/reearthx/idx"

type (
	EdgeExecution struct{}
)

func (EdgeExecution) Type() string { return "edgeExecution" }

type (
	EdgeExecutionID = idx.ID[EdgeExecution]
)

var (
	NewEdgeExecutionID = idx.New[EdgeExecution]
)
