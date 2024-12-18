package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqldataloader"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/usecasex"
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

	res, err := c.usecase.Fetch(ctx, jobIDs, getOperator(ctx))
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

	job, err := c.usecase.FindByID(ctx, id, getOperator(ctx))
	if err != nil {
		return nil, err
	}

	return gqlmodel.ToJob(job), nil
}

func (c *JobLoader) FindByWorkspace(ctx context.Context, wsID gqlmodel.ID, pagination *gqlmodel.Pagination) (*gqlmodel.JobConnection, error) {
	tid, err := gqlmodel.ToID[accountdomain.Workspace](wsID)
	if err != nil {
		return nil, err
	}

	res, pi, err := c.usecase.FindByWorkspace(ctx, tid, gqlmodel.ToPagination(pagination), getOperator(ctx))
	if err != nil {
		return nil, err
	}

	edges := make([]*gqlmodel.JobEdge, 0, len(res))
	nodes := make([]*gqlmodel.Job, 0, len(res))
	for _, j := range res {
		job := gqlmodel.ToJob(j)
		edges = append(edges, &gqlmodel.JobEdge{
			Node:   job,
			Cursor: usecasex.Cursor(job.ID),
		})
		nodes = append(nodes, job)
	}

	return &gqlmodel.JobConnection{
		Edges:      edges,
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
