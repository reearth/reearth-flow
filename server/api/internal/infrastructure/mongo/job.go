package mongo

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/rerror"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo/options"
)

var (
	jobIndexes       = []string{"deploymentid", "workspaceid", "status"}
	jobUniqueIndexes = []string{"id"}
)

type Job struct {
	client *mongox.ClientCollection
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
	return r
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

func (r *Job) FindByWorkspace(ctx context.Context, workspace accountdomain.WorkspaceID, pagination *interfaces.PaginationParam) ([]*job.Job, *interfaces.PageBasedInfo, error) {
	filter := bson.M{
		"workspaceid": workspace.String(),
		"$or": []bson.M{
			{"debug": false},
			{"debug": nil},
			{"debug": bson.M{"$exists": false}},
		},
	}

	total, err := r.client.Count(ctx, filter)
	if err != nil {
		return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
	}

	c := mongodoc.NewJobConsumer([]accountdomain.WorkspaceID{workspace})

	if pagination != nil && pagination.Page != nil {
		skip := int64((pagination.Page.Page - 1) * pagination.Page.PageSize)
		limit := int64(pagination.Page.PageSize)

		var sort bson.D
		if pagination.Page.OrderBy != nil {
			dir := 1
			if pagination.Page.OrderDir != nil && *pagination.Page.OrderDir == "DESC" {
				dir = -1
			}

			fieldNameMap := map[string]string{
				"startedAt":   "startedat",
				"completedAt": "completedat",
				"status":      "status",
				"id":          "id",
			}

			fieldName := *pagination.Page.OrderBy
			if mongoField, ok := fieldNameMap[fieldName]; ok {
				fieldName = mongoField
			}
			sort = bson.D{{Key: fieldName, Value: dir}}
		} else {
			sort = bson.D{{Key: "startedat", Value: -1}}
		}

		opts := options.Find().
			SetSkip(skip).
			SetLimit(limit).
			SetSort(sort)

		if err := r.client.Find(ctx, filter, c, opts); err != nil {
			return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
		}

		pageInfo := interfaces.NewPageBasedInfo(total, pagination.Page.Page, pagination.Page.PageSize)
		return c.Result, pageInfo, nil
	}

	if err := r.client.Find(ctx, filter, c); err != nil {
		return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
	}

	pageInfo := interfaces.NewPageBasedInfo(total, 1, int(total))
	return c.Result, pageInfo, nil
}

func (r *Job) CountByWorkspace(ctx context.Context, ws accountdomain.WorkspaceID) (int, error) {
	count, err := r.client.Count(ctx, bson.M{
		"workspaceid": ws.String(),
		"$or": []bson.M{
			{"debug": false},
			{"debug": nil},
			{"debug": bson.M{"$exists": false}},
		},
	})
	return int(count), err
}

func (r *Job) Save(ctx context.Context, j *job.Job) error {
	doc, id := mongodoc.NewJob(j)
	err := r.client.SaveOne(ctx, id, doc)
	return err
}

func (r *Job) Remove(ctx context.Context, id id.JobID) error {
	return r.client.RemoveOne(ctx, bson.M{"id": id.String()})
}

func (r *Job) find(ctx context.Context, filter interface{}) ([]*job.Job, error) {
	c := mongodoc.NewJobConsumer(nil)
	if err := r.client.Find(ctx, filter, c); err != nil {
		return nil, err
	}
	return c.Result, nil
}

func (r *Job) findOne(ctx context.Context, filter any) (*job.Job, error) {
	c := mongodoc.NewJobConsumer(nil)
	if err := r.client.FindOne(ctx, filter, c); err != nil {
		return nil, err
	}
	return c.Result[0], nil
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
