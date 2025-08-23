package userfacinglog

import (
	"encoding/json"
	"time"
)

type UserFacingLog struct {
	jobID     JobID
	timestamp time.Time
	message   string
	metadata  json.RawMessage
}

func NewUserFacingLog(jobID JobID, timestamp time.Time, message string, metadata json.RawMessage) *UserFacingLog {
	return &UserFacingLog{
		jobID:     jobID,
		timestamp: timestamp,
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

func (l *UserFacingLog) Message() string {
	return l.message
}

func (l *UserFacingLog) Metadata() json.RawMessage {
	return l.metadata
}
