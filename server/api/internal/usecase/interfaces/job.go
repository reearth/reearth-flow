package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/account/accountdomain"
)

type Job interface {
	Cancel(context.Context, id.JobID, *usecase.Operator) (*job.Job, error)
	Fetch(context.Context, []id.JobID, *usecase.Operator) ([]*job.Job, error)
	FindByID(context.Context, id.JobID, *usecase.Operator) (*job.Job, error)
	FindByWorkspace(context.Context, accountdomain.WorkspaceID, *PaginationParam, *usecase.Operator) ([]*job.Job, *PageBasedInfo, error)
	GetStatus(context.Context, id.JobID, *usecase.Operator) (job.Status, error)
	StartMonitoring(context.Context, *job.Job, *string, *usecase.Operator) error
	Subscribe(context.Context, id.JobID, *usecase.Operator) (chan job.Status, error)
	Unsubscribe(id.JobID, chan job.Status)
}
