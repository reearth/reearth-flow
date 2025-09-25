package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqldataloader"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type JobLoader struct {
	usecase interfaces.Job
}

func NewJobLoader(usecase interfaces.Job) *JobLoader {
	return &JobLoader{usecase: usecase}
}

func (c *JobLoader) Fetch(ctx context.Context, ids []gqlmodel.ID) ([]*gqlmodel.Job, []error) {
	jobIDs := make([]id.JobID, 0, len(ids))
	for _, gid := range ids {
		jid, err := id.JobIDFrom(string(gid))
		if err != nil {
			return nil, []error{err}
		}
		jobIDs = append(jobIDs, jid)
	}

	res, err := c.usecase.Fetch(ctx, jobIDs)
	if err != nil {
		return nil, []error{err}
	}

	jobs := make([]*gqlmodel.Job, 0, len(res))
	for _, job := range res {
		jobs = append(jobs, gqlmodel.ToJob(job))
	}

	return jobs, nil
}

func (c *JobLoader) FindByID(ctx context.Context, jobID gqlmodel.ID) (*gqlmodel.Job, error) {
	id, err := id.JobIDFrom(string(jobID))
	if err != nil {
		return nil, err
	}

	job, err := c.usecase.FindByID(ctx, id)
	if err != nil {
		return nil, err
	}

	return gqlmodel.ToJob(job), nil
}

func (c *JobLoader) FindByWorkspacePage(ctx context.Context, wsID gqlmodel.ID, pagination gqlmodel.PageBasedPagination) (*gqlmodel.JobConnection, error) {
	tid, err := gqlmodel.ToID[id.Workspace](wsID)
	if err != nil {
		return nil, err
	}

	paginationParam := gqlmodel.ToPageBasedPagination(pagination)

	res, pi, err := c.usecase.FindByWorkspace(ctx, tid, paginationParam)
	if err != nil {
		return nil, err
	}

	nodes := make([]*gqlmodel.Job, 0, len(res))
	for _, j := range res {
		nodes = append(nodes, gqlmodel.ToJob(j))
	}

	return &gqlmodel.JobConnection{
		Nodes:      nodes,
		PageInfo:   gqlmodel.ToPageInfo(pi),
		TotalCount: int(pi.TotalCount),
	}, nil
}

// data loaders

type JobDataLoader interface {
	Load(gqlmodel.ID) (*gqlmodel.Job, error)
	LoadAll([]gqlmodel.ID) ([]*gqlmodel.Job, []error)
}

func (c *JobLoader) DataLoader(ctx context.Context) JobDataLoader {
	return gqldataloader.NewJobLoader(gqldataloader.JobLoaderConfig{
		Wait:     dataLoaderWait,
		MaxBatch: dataLoaderMaxBatch,
		Fetch: func(keys []gqlmodel.ID) ([]*gqlmodel.Job, []error) {
			return c.Fetch(ctx, keys)
		},
	})
}

func (c *JobLoader) OrdinaryDataLoader(ctx context.Context) JobDataLoader {
	return &ordinaryJobLoader{
		fetch: func(keys []gqlmodel.ID) ([]*gqlmodel.Job, []error) {
			return c.Fetch(ctx, keys)
		},
	}
}

type ordinaryJobLoader struct {
	fetch func(keys []gqlmodel.ID) ([]*gqlmodel.Job, []error)
}

func (l *ordinaryJobLoader) Load(key gqlmodel.ID) (*gqlmodel.Job, error) {
	res, errs := l.fetch([]gqlmodel.ID{key})
	if len(errs) > 0 {
		return nil, errs[0]
	}
	if len(res) > 0 {
		return res[0], nil
	}
	return nil, nil
}

func (l *ordinaryJobLoader) LoadAll(keys []gqlmodel.ID) ([]*gqlmodel.Job, []error) {
	return l.fetch(keys)
}
