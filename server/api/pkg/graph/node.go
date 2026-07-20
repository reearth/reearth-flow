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

// FeaturesProcessed is the successfully processed feature count reported by
// a processor node on its terminal status event. Meaningful for processor
// nodes; sink nodes also report metrics but always leave this at 0, since
// the field does not apply to them. Nil means the node hasn't reached a
// terminal status yet, is a source node (sources never emit metrics), or
// the node execution predates this field. Do not infer node kind from
// nilness — 0 can mean "does not apply to this node kind" just as easily as
// "genuinely zero".
func (e *NodeExecution) FeaturesProcessed() *int {
	return e.featuresProcessed
}

// FeaturesWritten is the successfully written feature count reported by a
// sink node on its terminal status event. Meaningful for sink nodes;
// processor nodes also report metrics but always leave this at 0, since the
// field does not apply to them. Nil means the node hasn't reached a
// terminal status yet, is a source node (sources never emit metrics), or
// the node execution predates this field. Do not infer node kind from
// nilness — 0 can mean "does not apply to this node kind" just as easily as
// "genuinely zero".
func (e *NodeExecution) FeaturesWritten() *int {
	return e.featuresWritten
}

// FinishFeatureCount is the feature count a processor node emitted
// downstream during finish() (meaningful mainly for accumulating/aggregating
// actions). Meaningful for processor nodes; sink nodes also report metrics
// but always leave this at 0, since the field does not apply to them. Nil
// means the node hasn't reached a terminal status yet, is a source node
// (sources never emit metrics), or the node execution predates this field.
// Do not infer node kind from nilness — 0 can mean "does not apply to this
// node kind" just as easily as "genuinely zero".
func (e *NodeExecution) FinishFeatureCount() *int {
	return e.finishFeatureCount
}
