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

	return &UserFacingLog{
		JobID:     ID(d.JobID().String()),
		Timestamp: d.Timestamp(),
		Message:   d.Message(),
		Metadata:  metadata,
	}
}
