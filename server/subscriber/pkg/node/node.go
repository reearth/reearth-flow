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

type NodeStatusEvent struct {
	WorkflowID string    `json:"workflowId"`
	JobID      string    `json:"jobId"`
	NodeID     string    `json:"nodeId"`
	Status     Status    `json:"status"`
	FeatureID  *string   `json:"featureId,omitempty"`
	Timestamp  time.Time `json:"timestamp"`
}

type NodeExecution struct {
	ID          ID         `bson:"id"`
	JobID       string     `bson:"jobId"`
	NodeID      string     `bson:"edgeId"`
	Status      Status     `bson:"status"`
	StartedAt   *time.Time `bson:"startedAt,omitempty"`
	CompletedAt *time.Time `bson:"completedAt,omitempty"`
}
