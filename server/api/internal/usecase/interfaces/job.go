package interfaces

import (
	"context"
	"errors"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
)

// ErrJobAlreadyRunning refuses a run on a free workspace that already has as
// many pending or running jobs as its plan allows.
var ErrJobAlreadyRunning error = errors.New("this workspace's plan does not allow another job while one is running")

type Job interface {
	Cancel(context.Context, id.JobID) (*job.Job, error)
	Fetch(context.Context, []id.JobID) ([]*job.Job, error)
	FindByID(context.Context, id.JobID) (*job.Job, error)
	FindByWorkspace(context.Context, accountsid.WorkspaceID, *PaginationParam, *string) ([]*job.Job, *PageBasedInfo, error)
	GetStatus(context.Context, id.JobID) (job.Status, error)
	RunCloudRunWorker(j *job.Job, p gateway.RunJobParam)
	PreviewSchemaCloudRunWorker(j *job.Job, p gateway.ProbeSchemaParam)
	StartMonitoring(context.Context, *job.Job, *string) error
	Subscribe(context.Context, id.JobID) (chan job.Status, error)
	Unsubscribe(id.JobID, chan job.Status)
}
