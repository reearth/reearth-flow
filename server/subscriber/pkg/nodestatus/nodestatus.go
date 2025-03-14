package nodestatus

import (
	"errors"
	"time"
)

type NodeExecutionStatus string

const (
	NodeExecutionStatusPending   NodeExecutionStatus = "PENDING"
	NodeExecutionStatusRunning   NodeExecutionStatus = "RUNNING"
	NodeExecutionStatusSucceeded NodeExecutionStatus = "SUCCEEDED"
	NodeExecutionStatusFailed    NodeExecutionStatus = "FAILED"
)

var (
	ErrInvalidEdgePassEvent = errors.New("invalid edge pass event data")
)

type EventStatus string

const (
	EventStatusInProgress EventStatus = "inProgress"
	EventStatusCompleted  EventStatus = "completed"
)

type UpdatedEdge struct {
	ID        string      `json:"id"`
	Status    EventStatus `json:"status"`
	FeatureID *string     `json:"featureId,omitempty"`
}

type EdgePassThroughEvent struct {
	JobID        string        `json:"jobId"`
	Status       EventStatus   `json:"status"` 
	Timestamp    time.Time     `json:"timestamp"`
	UpdatedEdges []UpdatedEdge `json:"updatedEdges"`
	WorkflowID   string        `json:"workflowId"`
}

func NewEdgePassThroughEvent(
	jobID string,
	status EventStatus,
	timestamp time.Time,
	updatedEdges []UpdatedEdge,
	workflowID string,
) (*EdgePassThroughEvent, error) {
	if jobID == "" || workflowID == "" || len(updatedEdges) == 0 {
		return nil, ErrInvalidEdgePassEvent
	}

	return &EdgePassThroughEvent{
		JobID:        jobID,
		Status:       status,
		Timestamp:    timestamp,
		UpdatedEdges: updatedEdges,
		WorkflowID:   workflowID,
	}, nil
}
