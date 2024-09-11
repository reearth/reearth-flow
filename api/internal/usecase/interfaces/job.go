package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/usecasex"
)

type Job interface {
	Fetch(context.Context, []id.JobID, *usecase.Operator) ([]*job.Job, error)
	FindByID(context.Context, id.JobID, *usecase.Operator) (*job.Job, error)
	FindByWorkspace(context.Context, accountdomain.WorkspaceID, *usecasex.Pagination, *usecase.Operator) ([]*job.Job, *usecasex.PageInfo, error)
	GetStatus(context.Context, id.JobID, *usecase.Operator) (job.Status, error)
}
