package edge

import (
	"time"
)

type Status string

const (
	StatusInProgress Status = "inProgress"
	StatusCompleted  Status = "completed"
	StatusFailed     Status = "failed"
)

type PassThroughEvent struct {
	WorkflowID   string        `json:"workflowId"`
	JobID        string        `json:"jobId"`
	Status       Status        `json:"status"`
	Timestamp    time.Time     `json:"timestamp"`
	UpdatedEdges []UpdatedEdge `json:"updatedEdges"`
}

type UpdatedEdge struct {
	ID        string  `json:"id"`
	Status    Status  `json:"status"`
	FeatureID *string `json:"featureId,omitempty"`
}

type EdgeExecution struct {
	ID                  string     `bson:"id"`
	EdgeID              string     `bson:"edgeId"`
	Status              Status     `bson:"status"`
	StartedAt           *time.Time `bson:"startedAt,omitempty"`
	CompletedAt         *time.Time `bson:"completedAt,omitempty"`
	FeatureID           *string    `bson:"featureId,omitempty"`
	IntermediateDataURL string     `bson:"intermediateDataUrl,omitempty"`
}

type JobStatus string

const (
	JobStatusUnknown   JobStatus = "UNKNOWN"
	JobStatusCancelled JobStatus = "CANCELLED"
	JobStatusPending   JobStatus = "PENDING"
	JobStatusRunning   JobStatus = "RUNNING"
	JobStatusCompleted JobStatus = "COMPLETED"
	JobStatusFailed    JobStatus = "FAILED"
)
