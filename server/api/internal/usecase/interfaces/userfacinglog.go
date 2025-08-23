package interfaces

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/userfacinglog"
)

type UserFacingLog interface {
	GetUserFacingLogs(ctx context.Context, since time.Time, jobID id.JobID) ([]*userfacinglog.UserFacingLog, error)
	Subscribe(ctx context.Context, jobID id.JobID) (chan *userfacinglog.UserFacingLog, error)
	Unsubscribe(jobID id.JobID, ch chan *userfacinglog.UserFacingLog)
}
