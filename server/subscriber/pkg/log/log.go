package log

import (
	"errors"
	"time"
)

type LogLevel string

var ErrInvalidLogEvent = errors.New("invalid log event data")

const (
	LogLevelError LogLevel = "ERROR"
	LogLevelWarn  LogLevel = "WARN"
	LogLevelInfo  LogLevel = "INFO"
	LogLevelDebug LogLevel = "DEBUG"
	LogLevelTrace LogLevel = "TRACE"
)

type LogEvent struct {
	WorkflowID string    `json:"workflowId"`
	JobID      string    `json:"jobId"`
	NodeID     *string   `json:"nodeId,omitempty"` // Use pointer since nodeId may be null.
	Timestamp  time.Time `json:"timestamp"`
	LogLevel   LogLevel  `json:"logLevel"`
	Message    string    `json:"message"`
}

func NewLogEvent(
	workflowID string,
	jobID string,
	timestamp time.Time,
	logLevel LogLevel,
	message string,
	nodeID *string,
) (*LogEvent, error) {
	if workflowID == "" || jobID == "" {
		return nil, ErrInvalidLogEvent
	}

	return &LogEvent{
		WorkflowID: workflowID,
		JobID:      jobID,
		Timestamp:  timestamp,
		LogLevel:   logLevel,
		Message:    message,
		NodeID:     nodeID,
	}, nil
}
