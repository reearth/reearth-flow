package mongo

import (
	"context"

	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo/options"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/rerror"
)

var (
	projectIndexes       = []string{"alias", "alias,publishmentstatus", "workspace"}
	projectUniqueIndexes = []string{"id"}
)

type Project struct {
	client *mongox.ClientCollection
	f      repo.WorkspaceFilter
}

func NewProject(client *mongox.Client) *Project {
	return &Project{
		client: client.WithCollection("project"),
	}
}

func (r *Project) Init(ctx context.Context) error {
	return createIndexes(ctx, r.client, projectIndexes, projectUniqueIndexes)
}

func (r *Project) Filtered(f repo.WorkspaceFilter) repo.Project {
	return &Project{
		client: r.client,
		f:      r.f.Merge(f),
	}
}

func (r *Project) FindByID(ctx context.Context, id id.ProjectID) (*project.Project, error) {
	return r.findOne(ctx, bson.M{
		"id": id.String(),
	}, true)
}

func (r *Project) FindByIDs(ctx context.Context, ids id.ProjectIDList) ([]*project.Project, error) {
	if len(ids) == 0 {
		return nil, nil
	}

	filter := bson.M{
		"id": bson.M{
			"$in": ids.Strings(),
		},
	}
	res, err := r.find(ctx, filter)
	if err != nil {
		return nil, err
	}
	return filterProjects(ids, res), nil
}

func (r *Project) FindByWorkspace(ctx context.Context, id accountdomain.WorkspaceID, pagination *interfaces.PaginationParam) ([]*project.Project, *interfaces.PageBasedInfo, error) {
	if !r.f.CanRead(id) {
		return nil, interfaces.NewPageBasedInfo(0, 1, 1), nil
	}

	c := mongodoc.NewProjectConsumer(r.f.Readable)
	filter := bson.M{"workspace": id.String()}

	if pagination != nil && pagination.Page != nil {
		skip := int64((pagination.Page.Page - 1) * pagination.Page.PageSize)
		limit := int64(pagination.Page.PageSize)

		total, err := r.client.Count(ctx, filter)
		if err != nil {
			return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
		}

		sort := bson.D{{Key: "updatedat", Value: -1}}

		if pagination.Page.OrderBy != nil {
			sortDir := -1
			if pagination.Page.OrderDir != nil && *pagination.Page.OrderDir == "ASC" {
				sortDir = 1
			}

			fieldNameMap := map[string]string{
				"name":      "name",
				"createdAt": "createdat",
				"updatedAt": "updatedat",
				"status":    "status",
				"id":        "id",
			}

			fieldName := *pagination.Page.OrderBy
			if mongoField, ok := fieldNameMap[fieldName]; ok {
				fieldName = mongoField
			}
			sort = bson.D{{Key: fieldName, Value: sortDir}}
		}

		opts := options.Find().
			SetSort(sort).
			SetSkip(skip).
			SetLimit(limit)

		if err := r.client.Find(ctx, filter, c, opts); err != nil {
			return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
		}

		return c.Result, interfaces.NewPageBasedInfo(total, pagination.Page.Page, pagination.Page.PageSize), nil
	}

	if err := r.client.Find(ctx, filter, c); err != nil {
		return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
	}
	total := int64(len(c.Result))
	return c.Result, interfaces.NewPageBasedInfo(total, 1, len(c.Result)), nil
}

func (r *Project) FindByPublicName(ctx context.Context, name string) (*project.Project, error) {
	if name == "" {
		return nil, rerror.ErrNotFound
	}

	f := bson.D{
		{
			Key: "$or",
			Value: []bson.D{
				{{Key: "alias", Value: name}, {Key: "publishmentstatus", Value: bson.D{{Key: "$in", Value: []string{"public", "limited"}}}}},
				{{Key: "domains.domain", Value: name}, {Key: "publishmentstatus", Value: "public"}},
			},
		},
	}

	return r.findOne(ctx, f, false)
}

func (r *Project) CountByWorkspace(ctx context.Context, ws accountdomain.WorkspaceID) (int, error) {
	if !r.f.CanRead(ws) {
		return 0, repo.ErrOperationDenied
	}

	count, err := r.client.Count(ctx, bson.M{
		"workspace": ws.String(),
	})
	return int(count), err
}

func (r *Project) CountPublicByWorkspace(ctx context.Context, ws accountdomain.WorkspaceID) (int, error) {
	if !r.f.CanRead(ws) {
		return 0, repo.ErrOperationDenied
	}

	count, err := r.client.Count(ctx, bson.M{
		"workspace": ws.String(),
		"publishmentstatus": bson.M{
			"$in": []string{"public", "limited"},
		},
	})
	return int(count), err
}

func (r *Project) Save(ctx context.Context, project *project.Project) error {
	if !r.f.CanWrite(project.Workspace()) {
		return repo.ErrOperationDenied
	}
	doc, id := mongodoc.NewProject(project)
	return r.client.SaveOne(ctx, id, doc)
}

func (r *Project) Remove(ctx context.Context, id id.ProjectID) error {
	return r.client.RemoveOne(ctx, r.writeFilter(bson.M{"id": id.String()}))
}

func (r *Project) find(ctx context.Context, filter interface{}) ([]*project.Project, error) {
	c := mongodoc.NewProjectConsumer(r.f.Readable)
	if err := r.client.Find(ctx, filter, c); err != nil {
		return nil, err
	}
	return c.Result, nil
}

func (r *Project) findOne(ctx context.Context, filter any, filterByWorkspaces bool) (*project.Project, error) {
	var f []accountdomain.WorkspaceID
	if filterByWorkspaces {
		f = r.f.Readable
	}
	c := mongodoc.NewProjectConsumer(f)
	if err := r.client.FindOne(ctx, filter, c); err != nil {
		return nil, err
	}
	return c.Result[0], nil
}

func filterProjects(ids []id.ProjectID, rows []*project.Project) []*project.Project {
	res := make([]*project.Project, 0, len(ids))
	for _, id := range ids {
		var r2 *project.Project
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

// func (r *Project) readFilter(filter interface{}) interface{} {
// 	return applyWorkspaceFilter(filter, r.f.Readable)
// }

func (r *Project) writeFilter(filter interface{}) interface{} {
	return applyWorkspaceFilter(filter, r.f.Writable)
}
