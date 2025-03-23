package graph

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
)

type NodeExecutionBuilder struct {
	e *NodeExecution
}

func NewNodeExecutionBuilder() *NodeExecutionBuilder {
	return &NodeExecutionBuilder{e: &NodeExecution{}}
}

func (b *NodeExecutionBuilder) Build() (*NodeExecution, error) {
	if b.e.id.IsNil() {
		return nil, id.ErrInvalidID
	}
	return b.e, nil
}

func (b *NodeExecutionBuilder) MustBuild() *NodeExecution {
	r, err := b.Build()
	if err != nil {
		panic(err)
	}
	return r
}

func (b *NodeExecutionBuilder) ID(id NodeExecutionID) *NodeExecutionBuilder {
	b.e.id = id
	return b
}

func (b *NodeExecutionBuilder) JobID(jobID id.JobID) *NodeExecutionBuilder {
	b.e.jobID = jobID
	return b
}

func (b *NodeExecutionBuilder) NodeID(nodeID id.NodeID) *NodeExecutionBuilder {
	b.e.nodeID = nodeID
	return b
}

func (b *NodeExecutionBuilder) Status(status Status) *NodeExecutionBuilder {
	b.e.status = status
	return b
}

func (b *NodeExecutionBuilder) StartedAt(startedAt *time.Time) *NodeExecutionBuilder {
	b.e.startedAt = startedAt
	return b
}

func (b *NodeExecutionBuilder) CompletedAt(completedAt *time.Time) *NodeExecutionBuilder {
	b.e.completedAt = completedAt
	return b
}
