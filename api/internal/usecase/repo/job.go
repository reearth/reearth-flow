package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/usecasex"
	"github.com/samber/lo"
)

type Job interface {
	Filtered(WorkspaceFilter) Job
	FindByIDs(context.Context, id.JobIDList) ([]*job.Job, error)
	FindByID(context.Context, id.JobID) (*job.Job, error)
	FindByWorkspace(context.Context, accountdomain.WorkspaceID, *usecasex.Pagination) ([]*job.Job, *usecasex.PageInfo, error)
	Save(context.Context, *job.Job) error
	Remove(context.Context, id.JobID) error
}

func IterateJobsByWorkspace(repo Job, ctx context.Context, tid accountdomain.WorkspaceID, batch int64, callback func([]*job.Job) error) error {
	pagination := usecasex.CursorPagination{
		Before: nil,
		After:  nil,
		First:  lo.ToPtr(batch),
		Last:   nil,
	}.Wrap()

	for {
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

		if !info.HasNextPage {
			break
		}

		c := usecasex.Cursor(jobs[len(jobs)-1].ID().String())
		pagination.Cursor.After = &c
	}

	return nil
}
