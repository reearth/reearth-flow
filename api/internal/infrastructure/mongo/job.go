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

// Job represents a MongoDB repository implementation for managing jobs
type Job struct {
	client *mongox.ClientCollection
}

// NewJob creates a new Job repository instance
func NewJob(client *mongox.Client) repo.Job {
	return &Job{
		client: client.WithCollection("job"),
	}
}

// Init initializes the job collection by creating necessary indexes
func (r *Job) Init(ctx context.Context) error {
	return createIndexes(ctx, r.client, jobIndexes, jobUniqueIndexes)
}

// Filtered returns a filtered version of the job repository
func (r *Job) Filtered(f repo.WorkspaceFilter) repo.Job {
	return r
}

// FindByIDs retrieves multiple jobs by their IDs
// Returns nil if the ids list is empty
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

// FindByID retrieves a single job by its ID
func (r *Job) FindByID(ctx context.Context, id id.JobID) (*job.Job, error) {
	return r.findOne(ctx, bson.M{
		"id": id.String(),
	})
}

// FindByWorkspace retrieves jobs for a given workspace with pagination support
//
// Parameters:
//   - ctx: The context for the operation
//   - workspace: The workspace ID to filter jobs
//   - pagination: Optional pagination parameters
//   - Page: The page number (1-based indexing)
//   - PageSize: Number of items per page
//   - OrderBy: Field to sort by (supported fields: "startedAt", "completedAt")
//   - OrderDir: Sort direction ("ASC" or "DESC")
//
// Returns:
//   - []*job.Job: List of jobs for the given page
//   - *interfaces.PageBasedInfo: Pagination information including:
//   - TotalCount: Total number of jobs
//   - CurrentPage: Current page number
//   - TotalPages: Total number of pages
//   - error: Any error that occurred during the operation
//
// Example GraphQL Query:
//
//	{
//	  jobs(
//	    workspaceId: "xxx",
//	    pagination: {
//	      page: 1,
//	      pageSize: 10,
//	      orderBy: "startedAt",
//	      orderDir: DESC
//	    }
//	  ) {
//	    nodes {
//	      id
//	      status
//	      startedAt
//	      completedAt
//	    }
//	    pageInfo {
//	      totalCount
//	      currentPage
//	      totalPages
//	    }
//	  }
//	}
func (r *Job) FindByWorkspace(ctx context.Context, workspace accountdomain.WorkspaceID, pagination *interfaces.PaginationParam) ([]*job.Job, *interfaces.PageBasedInfo, error) {
	filter := bson.M{"workspaceid": workspace.String()}

	// Get total count for page info
	total, err := r.client.Count(ctx, filter)
	if err != nil {
		return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
	}

	// Create consumer with workspace filter
	c := mongodoc.NewJobConsumer([]accountdomain.WorkspaceID{workspace})

	if pagination != nil && pagination.Page != nil {
		// Page-based pagination
		skip := int64((pagination.Page.Page - 1) * pagination.Page.PageSize)
		limit := int64(pagination.Page.PageSize)

		// Set up sort options
		var sort bson.D
		if pagination.Page.OrderBy != nil {
			dir := 1
			if pagination.Page.OrderDir != nil && *pagination.Page.OrderDir == "DESC" {
				dir = -1
			}

			// Map GraphQL field names to MongoDB field names
			fieldNameMap := map[string]string{
				"startedAt":   "startedat",
				"completedAt": "completedat",
				"status":      "status",
				"id":          "id",
				// Add other field mappings here
			}

			fieldName := *pagination.Page.OrderBy
			if mongoField, ok := fieldNameMap[fieldName]; ok {
				fieldName = mongoField
			}
			sort = bson.D{{Key: fieldName, Value: dir}}
		} else {
			// Default sort by startedAt desc for better UX
			sort = bson.D{{Key: "startedat", Value: -1}}
		}

		// Find with pagination
		opts := options.Find().
			SetSkip(skip).
			SetLimit(limit).
			SetSort(sort)

		if err := r.client.Find(ctx, filter, c, opts); err != nil {
			return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
		}

		// Create page info
		pageInfo := interfaces.NewPageBasedInfo(total, pagination.Page.Page, pagination.Page.PageSize)
		return c.Result, pageInfo, nil
	}

	// No pagination
	if err := r.client.Find(ctx, filter, c); err != nil {
		return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
	}

	// Create page info without pagination
	pageInfo := interfaces.NewPageBasedInfo(total, 1, int(total))
	return c.Result, pageInfo, nil
}

// CountByWorkspace returns the total number of jobs in a workspace
func (r *Job) CountByWorkspace(ctx context.Context, ws accountdomain.WorkspaceID) (int, error) {
	count, err := r.client.Count(ctx, bson.M{
		"workspaceid": ws.String(),
	})
	return int(count), err
}

// Save persists a job to the database
// If the job already exists, it will be updated
func (r *Job) Save(ctx context.Context, j *job.Job) error {
	doc, id := mongodoc.NewJob(j)
	err := r.client.SaveOne(ctx, id, doc)
	return err
}

// Remove deletes a job from the database by its ID
func (r *Job) Remove(ctx context.Context, id id.JobID) error {
	return r.client.RemoveOne(ctx, bson.M{"id": id.String()})
}

// find is an internal helper method to find jobs based on a filter
func (r *Job) find(ctx context.Context, filter interface{}) ([]*job.Job, error) {
	c := mongodoc.NewJobConsumer(nil)
	if err := r.client.Find(ctx, filter, c); err != nil {
		return nil, err
	}
	return c.Result, nil
}

// findOne is an internal helper method to find a single job based on a filter
func (r *Job) findOne(ctx context.Context, filter any) (*job.Job, error) {
	c := mongodoc.NewJobConsumer(nil)
	if err := r.client.FindOne(ctx, filter, c); err != nil {
		return nil, err
	}
	return c.Result[0], nil
}

// filterJobs is an internal helper method to filter jobs by their IDs
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
