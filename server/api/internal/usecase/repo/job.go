package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
)

type Job interface {
	Filtered(WorkspaceFilter) Job
	FindByIDs(context.Context, id.JobIDList) ([]*job.Job, error)
	FindByID(context.Context, id.JobID) (*job.Job, error)
	FindByWorkspace(context.Context, id.WorkspaceID, *interfaces.PaginationParam) ([]*job.Job, *interfaces.PageBasedInfo, error)
	Save(context.Context, *job.Job) error
	Remove(context.Context, id.JobID) error
}

func IterateJobsByWorkspace(repo Job, ctx context.Context, tid id.WorkspaceID, batch int64, callback func([]*job.Job) error) error {
	page := 1
	for {
		pagination := &interfaces.PaginationParam{
			Page: &interfaces.PageBasedPaginationParam{
				Page:     page,
				PageSize: int(batch),
			},
		}

		jobs, info, err := repo.FindByWorkspace(ctx, tid, pagination)
		if err != nil {
			return err
		}
		if len(jobs) == 0 {
			break
		}

		if err := callback(jobs); err != nil {
			return err
		}

		if int64(info.CurrentPage*info.TotalPages) >= int64(page*int(batch)) {
			break
		}

		page++
	}
	return nil
}
