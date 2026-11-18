package repo

import (
	"context"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
)

type Job interface {
	Filtered(WorkspaceFilter) Job
	FindByIDs(context.Context, id.JobIDList) ([]*job.Job, error)
	FindByID(context.Context, id.JobID) (*job.Job, error)
	FindByWorkspace(context.Context, accountsid.WorkspaceID, *interfaces.PaginationParam, *string) ([]*job.Job, *interfaces.PageBasedInfo, error)
	FindByProject(context.Context, id.ProjectID) ([]*job.Job, error)
	// CountRunningByWorkspace counts the workspace's pending and running debug
	// runs started at or after since. Deployment and trigger runs and
	// preview-schema probes are not counted, as the plan caps only debug runs
	// at a time.
	CountRunningByWorkspace(ctx context.Context, ws accountsid.WorkspaceID, since time.Time) (int, error)
	// FindRunningByWorkspace returns the jobs CountRunningByWorkspace counts.
	FindRunningByWorkspace(ctx context.Context, ws accountsid.WorkspaceID, since time.Time) ([]*job.Job, error)
	// FindByScheduledRun returns the job a time-driven trigger started for one
	// scheduled time, or rerror.ErrNotFound.
	FindByScheduledRun(ctx context.Context, run job.ScheduledRun) (*job.Job, error)
	// SaveIfRunningBelow atomically saves a new job only while its workspace
	// has fewer than limit jobs as CountRunningByWorkspace counts them, across
	// all server instances. Otherwise it returns interfaces.ErrJobAlreadyRunning,
	// and like Save it returns ErrScheduledRunInProgress for a taken scheduled run.
	SaveIfRunningBelow(ctx context.Context, j *job.Job, limit int, since time.Time) error
	RemoveByProject(context.Context, id.ProjectID) error
	// Save saves a job, and returns interfaces.ErrScheduledRunInProgress when
	// another job already has its scheduled run.
	Save(context.Context, *job.Job) error
	Remove(context.Context, id.JobID) error
}

func IterateJobsByWorkspace(repo Job, ctx context.Context, tid accountsid.WorkspaceID, batch int64, callback func([]*job.Job) error) error {
	page := 1
	for {
		pagination := &interfaces.PaginationParam{
			Page: &interfaces.PageBasedPaginationParam{
				Page:     page,
				PageSize: int(batch),
			},
		}

		jobs, info, err := repo.FindByWorkspace(ctx, tid, pagination, nil)
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
