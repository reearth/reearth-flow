package gateway

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
)

type Log interface {
	GetLogs(context.Context, time.Time, id.WorkflowID, id.JobID) ([]*log.Log, error)
}
