package mongo

import (
	"context"
	"errors"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/rerror"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
	"go.mongodb.org/mongo-driver/x/mongo/driver"
)

var (
	jobIndexes       = []string{"deploymentid", "workspaceid", "projectid", "status", "workspaceid,startedat"}
	jobUniqueIndexes = []string{"id"}
)

type Job struct {
	client *mongox.ClientCollection
	// admission holds one document per workspace whose version every job
	// admitted by SaveIfRunningBelow increments.
	admission *mongox.ClientCollection
}

func NewJob(client *mongox.Client) repo.Job {
	return &Job{
		client:    client.WithCollection("job"),
		admission: client.WithCollection("jobAdmission"),
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

func (r *Job) FindByWorkspace(ctx context.Context, workspace accountsid.WorkspaceID, pagination *interfaces.PaginationParam, keyword *string) ([]*job.Job, *interfaces.PageBasedInfo, error) {
	filter := bson.M{
		"workspaceid": workspace.String(),
		"$or": []bson.M{
			{"debug": false},
			{"debug": nil},
			{"debug": bson.M{"$exists": false}},
		},
	}

	if keyword != nil && *keyword != "" {
		filter = bson.M{
			"workspaceid": workspace.String(),
			"$and": []bson.M{
				{
					"$or": []bson.M{
						{"debug": false},
						{"debug": nil},
						{"debug": bson.M{"$exists": false}},
					},
				},
				{
					"$or": []bson.M{
						{"id": bson.M{"$regex": *keyword, "$options": "i"}},
						{"status": bson.M{"$regex": *keyword, "$options": "i"}},
					},
				},
			},
		}
	}

	total, err := r.client.Count(ctx, filter)
	if err != nil {
		return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
	}

	c := mongodoc.NewJobConsumer([]accountsid.WorkspaceID{workspace})

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

func (r *Job) CountByWorkspace(ctx context.Context, ws accountsid.WorkspaceID) (int, error) {
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

func (r *Job) CountRunningByWorkspace(ctx context.Context, ws accountsid.WorkspaceID, since time.Time) (int, error) {
	// Legacy docs predate the mode field, so a missing mode counts as a run.
	count, err := r.client.Count(ctx, bson.M{
		"workspaceid": ws.String(),
		"startedat":   bson.M{"$gte": since},
		"status": bson.M{
			"$in": []string{string(job.StatusPending), string(job.StatusRunning)},
		},
		"mode": bson.M{
			"$ne": string(job.ModePreviewSchema),
		},
	})
	if err != nil {
		return 0, rerror.ErrInternalByWithContext(ctx, err)
	}
	return int(count), nil
}

func (r *Job) FindByProject(ctx context.Context, projectID id.ProjectID) ([]*job.Job, error) {
	// Exclude preview-schema jobs from run history. They are persisted (debug=true)
	// but are not real runs, so they must not pollute the project's job listing.
	// Legacy docs predate the mode field, so treat missing/empty mode as a run.
	filter := bson.M{
		"projectid": projectID.String(),
		"debug":     true,
		"mode": bson.M{
			"$ne": string(job.ModePreviewSchema),
		},
	}

	return r.find(ctx, filter)
}

func (r *Job) RemoveByProject(ctx context.Context, projectID id.ProjectID) error {
	return r.client.RemoveAll(ctx, bson.M{
		"projectid": projectID.String(),
		"debug":     true,
	})
}

// jobAdmissionAttempts bounds how often SaveIfRunningBelow retries after losing
// a race. Every lost race means another job was admitted, so running out of
// attempts is reported as the workspace being busy.
const jobAdmissionAttempts = 3

// SaveIfRunningBelow works with and without transactions. It reads the
// workspace's admission version, counts, saves the job, and then moves the
// version on only if it is still the one it read. A concurrent call that
// admitted a job in between has moved it already, so this call withdraws its
// job and counts again, now seeing the other job. Each admitted job therefore
// counted every job admitted before it.
//
// Inside a transaction a lost race aborts the transaction (a write conflict or
// duplicate key on the version document, or a version that moved), so the run
// is refused and the rollback withdraws the job.
func (r *Job) SaveIfRunningBelow(ctx context.Context, j *job.Job, limit int, since time.Time) error {
	ws := j.Workspace().String()
	inTx := mongo.SessionFromContext(ctx) != nil
	for attempt := 0; attempt < jobAdmissionAttempts; attempt++ {
		v, err := r.admissionVersion(ctx, ws)
		if err != nil {
			return err
		}
		n, err := r.CountRunningByWorkspace(ctx, j.Workspace(), since)
		if err != nil {
			return err
		}
		if n >= limit {
			return interfaces.ErrJobAlreadyRunning
		}
		if err := r.Save(ctx, j); err != nil {
			return err
		}
		admitted, err := r.bumpAdmission(ctx, ws, v)
		if err != nil {
			if !inTx {
				_ = r.Remove(ctx, j.ID())
			}
			return rerror.ErrInternalByWithContext(ctx, err)
		}
		if admitted {
			return nil
		}
		if inTx {
			return interfaces.ErrJobAlreadyRunning
		}
		if err := r.Remove(ctx, j.ID()); err != nil {
			return err
		}
	}
	return interfaces.ErrJobAlreadyRunning
}

// admissionVersion returns 0 for a workspace that has never admitted a job.
func (r *Job) admissionVersion(ctx context.Context, ws string) (int64, error) {
	var doc struct {
		V int64 `bson:"v"`
	}
	err := r.admission.Client().FindOne(ctx, bson.M{"_id": ws}).Decode(&doc)
	if errors.Is(err, mongo.ErrNoDocuments) {
		return 0, nil
	}
	if err != nil {
		return 0, rerror.ErrInternalByWithContext(ctx, err)
	}
	return doc.V, nil
}

// bumpAdmission moves the version from v to v+1 and reports whether it was
// still v. The first admission creates the document, and the unique _id makes
// a concurrent first admission lose the same way. A write conflict inside a
// transaction is a lost race too.
func (r *Job) bumpAdmission(ctx context.Context, ws string, v int64) (bool, error) {
	var err error
	matched := true
	if v == 0 {
		_, err = r.admission.Client().InsertOne(ctx, bson.M{"_id": ws, "v": int64(1)})
	} else {
		var res *mongo.UpdateResult
		res, err = r.admission.Client().UpdateOne(ctx, bson.M{"_id": ws, "v": v}, bson.M{"$inc": bson.M{"v": 1}})
		matched = err == nil && res.MatchedCount == 1
	}
	if mongo.IsDuplicateKeyError(err) || isTransientTxError(err) {
		return false, nil
	}
	if err != nil {
		return false, err
	}
	return matched, nil
}

func isTransientTxError(err error) bool {
	var le mongo.LabeledError
	return errors.As(err, &le) && le.HasErrorLabel(driver.TransientTransactionError)
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
