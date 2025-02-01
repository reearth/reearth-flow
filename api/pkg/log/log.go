package log

import "time"

type Level string

const (
	LevelError Level = "ERROR"
	LevelWarn  Level = "WARN"
	LevelInfo  Level = "INFO"
	LevelDebug Level = "DEBUG"
	LevelTrace Level = "TRACE"
)

type Log struct {
	jobID     JobID
	nodeID    *NodeID
	timestamp time.Time
	level     Level
	message   string
}

func NewLog(jobID JobID, nodeID *NodeID, time time.Time, level Level, message string) *Log {
	return &Log{
		jobID:     jobID,
		nodeID:    nodeID,
		timestamp: time,
		level:     level,
		message:   message,
	}
}

func (l *Log) JobID() JobID {
	return l.jobID
}
func (l *Log) NodeID() *NodeID {
	return l.nodeID
}
func (l *Log) Timestamp() time.Time {
	return l.timestamp
}
func (l *Log) Level() Level {
	return l.level
}
func (l *Log) Message() string {
	return l.message
}
