package userfacinglog

import (
	"encoding/json"
	"time"
)

type LogLevel string

const (
	LogLevelInfo    LogLevel = "INFO"
	LogLevelSuccess LogLevel = "SUCCESS"
	LogLevelError   LogLevel = "ERROR"
)

type UserFacingLog struct {
	timestamp time.Time
	nodeID    *string
	nodeName  *string
	level     LogLevel
	message   string
	metadata  json.RawMessage
	jobID     JobID
}

func NewUserFacingLog(jobID JobID, timestamp time.Time, message string, metadata json.RawMessage) *UserFacingLog {
	return &UserFacingLog{
		jobID:     jobID,
		timestamp: timestamp,
		level:     LogLevelInfo, // Default level
		message:   message,
		metadata:  metadata,
	}
}

func NewUserFacingLogWithDetails(
	jobID JobID,
	timestamp time.Time,
	level LogLevel,
	nodeID *string,
	nodeName *string,
	message string,
	metadata json.RawMessage,
) *UserFacingLog {
	return &UserFacingLog{
		jobID:     jobID,
		timestamp: timestamp,
		level:     level,
		nodeID:    nodeID,
		nodeName:  nodeName,
		message:   message,
		metadata:  metadata,
	}
}

func (l *UserFacingLog) JobID() JobID {
	return l.jobID
}

func (l *UserFacingLog) Timestamp() time.Time {
	return l.timestamp
}

func (l *UserFacingLog) Level() LogLevel {
	return l.level
}

func (l *UserFacingLog) NodeID() *string {
	return l.nodeID
}

func (l *UserFacingLog) NodeName() *string {
	return l.nodeName
}

func (l *UserFacingLog) Message() string {
	return l.message
}

func (l *UserFacingLog) Metadata() json.RawMessage {
	return l.metadata
}
