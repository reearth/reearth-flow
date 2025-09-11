package gqlmodel

import (
	"encoding/json"

	"github.com/reearth/reearth-flow/api/pkg/userfacinglog"
)

func ToUserFacingLog(d *userfacinglog.UserFacingLog) *UserFacingLog {
	if d == nil {
		return nil
	}

	var metadata JSON
	if d.Metadata() != nil && len(d.Metadata()) > 0 {
		var m map[string]interface{}
		if err := json.Unmarshal(d.Metadata(), &m); err == nil {
			metadata = JSON(m)
		}
	}

	// Map LogLevel to GraphQL enum
	var level UserFacingLogLevel
	switch d.Level() {
	case userfacinglog.LogLevelInfo:
		level = UserFacingLogLevelInfo
	case userfacinglog.LogLevelSuccess:
		level = UserFacingLogLevelSuccess
	case userfacinglog.LogLevelError:
		level = UserFacingLogLevelError
	default:
		level = UserFacingLogLevelInfo
	}

	// Convert node ID and name
	var nodeID *ID
	if d.NodeID() != nil {
		id := ID(*d.NodeID())
		nodeID = &id
	}

	return &UserFacingLog{
		JobID:     ID(d.JobID().String()),
		Timestamp: d.Timestamp(),
		Level:     level,
		NodeID:    nodeID,
		NodeName:  d.NodeName(),
		Message:   d.Message(),
		Metadata:  metadata,
	}
}
