package node

import (
	"time"
)

type Status string

const (
	StatusPending    Status = "PENDING"
	StatusStarting   Status = "STARTING"
	StatusProcessing Status = "PROCESSING"
	StatusCompleted  Status = "COMPLETED"
	StatusFailed     Status = "FAILED"
)

type NodeMetrics struct {
	FeaturesProcessed  uint64 `json:"featuresProcessed" bson:"featuresProcessed"`
	FeaturesWritten    uint64 `json:"featuresWritten" bson:"featuresWritten"`
	FinishFeatureCount uint64 `json:"finishFeatureCount" bson:"finishFeatureCount"`
}

type NodeStatusEvent struct {
	WorkflowID string       `json:"workflowId"`
	JobID      string       `json:"jobId"`
	NodeID     string       `json:"nodeId"`
	Status     Status       `json:"status"`
	FeatureID  *string      `json:"featureId,omitempty"`
	Timestamp  time.Time    `json:"timestamp"`
	Metrics    *NodeMetrics `json:"metrics,omitempty"`
}

type NodeExecution struct {
	ID          string       `bson:"id"`
	JobID       string       `bson:"jobId"`
	NodeID      string       `bson:"nodeId"`
	Status      Status       `bson:"status"`
	StartedAt   *time.Time   `bson:"startedAt,omitempty"`
	CompletedAt *time.Time   `bson:"completedAt,omitempty"`
	Metrics     *NodeMetrics `bson:"metrics,omitempty"`
}
