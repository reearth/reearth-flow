package interfaces

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
)

type Log interface {
	GetLogs(context.Context, time.Time, id.JobID) ([]*log.Log, error)
	Subscribe(context.Context, id.JobID) (chan *log.Log, error)
	Unsubscribe(id.JobID, chan *log.Log)
}
