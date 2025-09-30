package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
)

type Job interface {
	Cancel(context.Context, id.JobID) (*job.Job, error)
	Fetch(context.Context, []id.JobID) ([]*job.Job, error)
	FindByID(context.Context, id.JobID) (*job.Job, error)
	FindByWorkspace(context.Context, id.WorkspaceID, *PaginationParam) ([]*job.Job, *PageBasedInfo, error)
	GetStatus(context.Context, id.JobID) (job.Status, error)
	StartMonitoring(context.Context, *job.Job, *string) error
	Subscribe(context.Context, id.JobID) (chan job.Status, error)
	Unsubscribe(id.JobID, chan job.Status)
}
