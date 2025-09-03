package userfacinglog

import (
	"errors"
	"time"
)

type UserFacingLogLevel string

const (
	UserFacingLogLevelInfo    UserFacingLogLevel = "info"
	UserFacingLogLevelSuccess UserFacingLogLevel = "success"
	UserFacingLogLevelError   UserFacingLogLevel = "error"
)

type UserFacingLogEvent struct {
	WorkflowID     string             `json:"workflowId"`
	JobID          string             `json:"jobId"`
	Timestamp      time.Time          `json:"timestamp"`
	Level          UserFacingLogLevel `json:"level"`
	NodeName       *string            `json:"nodeName,omitempty"`
	NodeID         *string            `json:"nodeId,omitempty"`
	DisplayMessage string             `json:"displayMessage"`
}

var ErrInvalidUserFacingLogEvent = errors.New("invalid user facing log event data")

func NewUserFacingLogEvent(
	workflowID string,
	jobID string,
	timestamp time.Time,
	level UserFacingLogLevel,
	nodeName *string,
	nodeID *string,
	displayMessage string,
) (*UserFacingLogEvent, error) {
	if workflowID == "" || jobID == "" {
		return nil, ErrInvalidUserFacingLogEvent
	}

	return &UserFacingLogEvent{
		WorkflowID:     workflowID,
		JobID:          jobID,
		Timestamp:      timestamp,
		Level:          level,
		NodeName:       nodeName,
		NodeID:         nodeID,
		DisplayMessage: displayMessage,
	}, nil
}
