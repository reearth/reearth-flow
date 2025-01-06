package mongo

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
	"go.mongodb.org/mongo-driver/bson"
)

var (
	jobIndexes       = []string{"deploymentid", "workspaceid", "status"}
	jobUniqueIndexes = []string{"id"}
)

type Job struct {
	client *mongox.ClientCollection
	f      repo.WorkspaceFilter
}

func NewJob(client *mongox.Client) repo.Job {
	return &Job{
		client: client.WithCollection("job"),
	}
}

func (r *Job) Init(ctx context.Context) error {
	return createIndexes(ctx, r.client, jobIndexes, jobUniqueIndexes)
}

func (r *Job) Filtered(f repo.WorkspaceFilter) repo.Job {
	return &Job{
		client: r.client,
		f:      r.f.Merge(f),
	}
}

func (r *Job) FindByIDs(ctx context.Context, ids id.JobIDList) ([]*job.Job, error) {
	if len(ids) == 0 {
		return nil, nil
	}

	// Convert JobIDs to strings for MongoDB query
	idStrings := make([]string, len(ids))
	for i, id := range ids {
		idStrings[i] = id.String()
	}

	filter := bson.M{
		"id": bson.M{
			"$in": idStrings,
		},
	}
	res, err := r.find(ctx, filter)
	if err != nil {
		return nil, err
	}
	return filterJobs(ids, res), nil
}

func (r *Job) FindByID(ctx context.Context, id id.JobID) (*job.Job, error) {
	return r.findOne(ctx, bson.M{
		"id": id.String(),
	})
}

func (r *Job) FindByWorkspace(ctx context.Context, workspace accountdomain.WorkspaceID, pagination *usecasex.Pagination) ([]*job.Job, *usecasex.PageInfo, error) {
	if !r.f.CanRead(workspace) {
		return nil, usecasex.EmptyPageInfo(), nil
	}
	return r.paginate(ctx, bson.M{
		"workspaceid": workspace.String(),
	}, pagination)
}

func (r *Job) CountByWorkspace(ctx context.Context, ws accountdomain.WorkspaceID) (int, error) {
	if !r.f.CanRead(ws) {
		return 0, nil
	}
	count, err := r.client.Count(ctx, bson.M{
		"workspaceid": ws.String(),
	})
	return int(count), err
}

func (r *Job) Save(ctx context.Context, j *job.Job) error {
	if !r.f.CanWrite(j.Workspace()) {
		return repo.ErrOperationDenied
	}
	doc, id := mongodoc.NewJob(j)

	err := r.client.SaveOne(ctx, id, doc)
	return err
}

func (r *Job) Remove(ctx context.Context, id id.JobID) error {
	return r.client.RemoveOne(ctx, r.writeFilter(bson.M{"id": id.String()}))
}

func (r *Job) find(ctx context.Context, filter interface{}) ([]*job.Job, error) {
	c := mongodoc.NewJobConsumer(r.f.Readable)
	if err := r.client.Find(ctx, filter, c); err != nil {
		return nil, err
	}
	return c.Result, nil
}

func (r *Job) findOne(ctx context.Context, filter any) (*job.Job, error) {
	c := mongodoc.NewJobConsumer(r.f.Readable)
	if err := r.client.FindOne(ctx, filter, c); err != nil {
		return nil, err
	}
	return c.Result[0], nil
}

func (r *Job) paginate(ctx context.Context, filter bson.M, pagination *usecasex.Pagination) ([]*job.Job, *usecasex.PageInfo, error) {
	c := mongodoc.NewJobConsumer(r.f.Readable)
	pageInfo, err := r.client.Paginate(ctx, filter, nil, pagination, c)
	if err != nil {
		return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
	}
	return c.Result, pageInfo, nil
}

func filterJobs(ids []id.JobID, rows []*job.Job) []*job.Job {
	res := make([]*job.Job, 0, len(ids))
	for _, id := range ids {
		var r2 *job.Job
		for _, r := range rows {
			if r.ID() == id {
				r2 = r
				break
			}
		}
		res = append(res, r2)
	}
	return res
}

func (r *Job) writeFilter(filter interface{}) interface{} {
	return applyWorkspaceFilter(filter, r.f.Writable)
}
