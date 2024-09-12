package interactor

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/usecasex"
)

type Job struct {
	common
	jobRepo        repo.Job
	deploymentRepo repo.Deployment
	workspaceRepo  accountrepo.Workspace
	transaction    usecasex.Transaction
	batch          gateway.Batch
}

func NewJob(r *repo.Container, gr *gateway.Container) interfaces.Job {
	return &Job{
		jobRepo:        r.Job,
		deploymentRepo: r.Deployment,
		workspaceRepo:  r.Workspace,
		transaction:    r.Transaction,
		batch:          gr.Batch,
	}
}

func (i *Job) FindByID(ctx context.Context, id id.JobID, operator *usecase.Operator) (*job.Job, error) {
	j, err := i.jobRepo.FindByID(ctx, id)
	if err != nil {
		return nil, err
	}
	if err := i.CanReadWorkspace(j.Workspace(), operator); err != nil {
		return nil, err
	}
	return j, nil
}

func (i *Job) Fetch(ctx context.Context, ids []id.JobID, operator *usecase.Operator) ([]*job.Job, error) {
	jobs, err := i.jobRepo.FindByIDs(ctx, ids)
	if err != nil {
		return nil, err
	}
	return i.filterReadableJobs(jobs, operator), nil
}

func (i *Job) FindByWorkspace(ctx context.Context, wsID accountdomain.WorkspaceID, pagination *usecasex.Pagination, operator *usecase.Operator) ([]*job.Job, *usecasex.PageInfo, error) {
	if err := i.CanReadWorkspace(wsID, operator); err != nil {
		return nil, nil, err
	}
	return i.jobRepo.FindByWorkspace(ctx, wsID, pagination)
}

func (i *Job) GetStatus(ctx context.Context, jobID id.JobID, operator *usecase.Operator) (job.Status, error) {
	j, err := i.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		return "", err
	}
	if err := i.CanReadWorkspace(j.Workspace(), operator); err != nil {
		return "", err
	}
	return j.Status(), nil
}

func (i *Job) filterReadableJobs(jobs []*job.Job, operator *usecase.Operator) []*job.Job {
	result := make([]*job.Job, 0, len(jobs))
	for _, j := range jobs {
		if i.CanReadWorkspace(j.Workspace(), operator) == nil {
			result = append(result, j)
		}
	}
	return result
}
