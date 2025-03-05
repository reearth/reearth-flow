package gqlmodel

import (
	"github.com/reearth/reearth-flow/api/pkg/log"
)

func ToLog(d *log.Log) *Log {
	if d == nil {
		return nil
	}
	var nodeID *ID
	if d.NodeID() != nil {
		id := ID(d.NodeID().String())
		nodeID = &id
	}
	return &Log{
		JobID:     ID(d.JobID().String()),
		NodeID:    nodeID,
		Timestamp: d.Timestamp(),
		LogLevel:  LogLevel(d.Level()),
		Message:   d.Message(),
	}
}
