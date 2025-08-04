package job

import (
	"time"
)

type Status string

const (
	StatusPending   Status = "PENDING"
	StatusStarting  Status = "STARTING"
	StatusRunning   Status = "RUNNING"
	StatusCompleted Status = "COMPLETED"
	StatusFailed    Status = "FAILED"
	StatusCancelled Status = "CANCELLED"
)

type JobStatusEvent struct {
	WorkflowID  string    `json:"workflowId"`
	JobID       string    `json:"jobId"`
	Status      Status    `json:"status"`
	Message     *string   `json:"message,omitempty"`
	FailedNodes *[]string `json:"failedNodes,omitempty"`
	Timestamp   time.Time `json:"timestamp"`
}

type Job struct {
	ID          string     `bson:"id"`
	WorkflowID  string     `bson:"workflowId"`
	Status      Status     `bson:"status"`
	Message     *string    `bson:"message,omitempty"`
	FailedNodes []string   `bson:"failedNodes,omitempty"`
	StartedAt   *time.Time `bson:"startedAt,omitempty"`
	CompletedAt *time.Time `bson:"completedAt,omitempty"`
	UpdatedAt   time.Time  `bson:"updatedAt"`
}
