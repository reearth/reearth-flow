package stdoutlog

import (
	"time"

	log "github.com/reearth/reearth-flow/subscriber/pkg/log"
)

type Event struct {
	WorkflowID string       `json:"workflowId"`
	JobID      string       `json:"jobId"`
	Timestamp  time.Time    `json:"timestamp"`
	LogLevel   log.LogLevel `json:"logLevel"`
	Message    string       `json:"message"`
	Target     string       `json:"target"`
}

func NewEvent(workflowID, jobID string, logLevel log.LogLevel, message, target string, timestamp time.Time) *Event {
	return &Event{
		WorkflowID: workflowID,
		JobID:      jobID,
		Timestamp:  timestamp,
		LogLevel:   logLevel,
		Message:    message,
		Target:     target,
	}
}
