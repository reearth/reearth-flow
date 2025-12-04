package graph

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
)

type Status string

const (
	StatusPending    Status = "PENDING"
	StatusStarting   Status = "STARTING"
	StatusProcessing Status = "PROCESSING"
	StatusCompleted  Status = "COMPLETED"
	StatusFailed     Status = "FAILED"
)

type NodeExecution struct {
	startedAt   *time.Time
	completedAt *time.Time
	id          string
	status      Status
	jobID       id.JobID
	nodeID      id.NodeID
}

func NewNodeExecution(
	id string,
	jobID id.JobID,
	nodeID id.NodeID,
	status Status,
) *NodeExecution {
	return &NodeExecution{
		id:     id,
		jobID:  jobID,
		nodeID: nodeID,
		status: status,
	}
}

func (e *NodeExecution) ID() string {
	return e.id
}

func (e *NodeExecution) JobID() id.JobID {
	return e.jobID
}

func (e *NodeExecution) NodeID() id.NodeID {
	return e.nodeID
}

func (e *NodeExecution) Status() Status {
	return e.status
}

func (e *NodeExecution) StartedAt() *time.Time {
	return e.startedAt
}

func (e *NodeExecution) CompletedAt() *time.Time {
	return e.completedAt
}
