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
	workflowID WorkflowID
	jobID      JobID
	nodeID     *NodeID
	timestamp  time.Time
	level      Level
	message    string
}

func NewLog(workflowID WorkflowID, jobID JobID, nodeID *NodeID, level Level, message string) *Log {
	return &Log{
		workflowID: workflowID,
		jobID:      jobID,
		nodeID:     nodeID,
		timestamp:  time.Now().UTC(),
		level:      level,
		message:    message,
	}
}

func (l *Log) WorkflowID() WorkflowID {
	return l.workflowID
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
