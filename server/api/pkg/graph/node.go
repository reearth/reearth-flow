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
	startedAt          *time.Time
	completedAt        *time.Time
	featuresProcessed  *int
	featuresWritten    *int
	finishFeatureCount *int
	id                 string
	status             Status
	jobID              id.JobID
	nodeID             id.NodeID
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

// FeaturesProcessed is the successfully processed feature count from a
// processor node's terminal status event; always 0 for sink nodes (does not
// apply). Nil means not-yet-terminal, a source node, or predates this
// field — do not infer node kind from a 0 value.
func (e *NodeExecution) FeaturesProcessed() *int {
	return e.featuresProcessed
}

// FeaturesWritten is the successfully written feature count from a sink
// node's terminal status event; always 0 for processor nodes (does not
// apply). Nil means not-yet-terminal, a source node, or predates this
// field — do not infer node kind from a 0 value.
func (e *NodeExecution) FeaturesWritten() *int {
	return e.featuresWritten
}

// FinishFeatureCount is the feature count a processor node emitted
// downstream during finish() (mainly for accumulating/aggregating actions);
// always 0 for sink nodes. Nil means not-yet-terminal, a source node, or
// predates this field — do not infer node kind from a 0 value.
func (e *NodeExecution) FinishFeatureCount() *int {
	return e.finishFeatureCount
}
