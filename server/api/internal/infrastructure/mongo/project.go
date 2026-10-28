package mongo

import (
	"context"
	"errors"
	"fmt"
	"regexp"
	"strings"

	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/bson/primitive"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/rerror"
)

// "workspace,alias" is listed as unique so createIndexes counts the partial
// index made in Init as its own and never builds a plain one, which would fail
// on documents saved before aliases existed: they all hold an empty alias.
var (
	projectIndexes       = []string{"alias", "alias,publishmentstatus", "workspace", "workspace,createdby"}
	projectUniqueIndexes = []string{"id", "workspace,alias"}
)

const projectAliasIndex = "workspace_alias_unique"

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
	if _, err := r.client.Client().Indexes().CreateOne(ctx, mongo.IndexModel{
		Keys: bson.D{{Key: "workspace", Value: 1}, {Key: "alias", Value: 1}},
		Options: options.Index().
			SetName(projectAliasIndex).
			SetUnique(true).
			SetPartialFilterExpression(bson.M{"alias": bson.M{"$gt": ""}}),
	}); err != nil {
		return err
	}
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

func (r *Project) FindByAlias(ctx context.Context, ws accountsid.WorkspaceID, alias string) (*project.Project, error) {
	if !r.f.CanRead(ws) {
		return nil, rerror.ErrNotFound
	}
	filter := bson.M{"workspace": ws.String(), "alias": alias}
	// A document saved before aliases existed has no alias stored and reads
	// back with its default one, so look that up by ID.
	if pid, ok := strings.CutPrefix(alias, project.DefaultAliasPrefix); ok {
		filter = bson.M{
			"workspace": ws.String(),
			"$or": []bson.M{
				{"alias": alias},
				{"id": pid, "alias": bson.M{"$in": []any{"", nil}}},
			},
		}
	}
	return r.findOne(ctx, filter, true)
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

func (r *Project) FindByWorkspace(ctx context.Context, id accountsid.WorkspaceID, pagination *interfaces.PaginationParam, keyword *string, includeArchived *bool, createdBy accountsid.UserIDList) ([]*project.Project, *interfaces.PageBasedInfo, error) {
	if !r.f.CanRead(id) {
		return nil, interfaces.NewPageBasedInfo(0, 1, 1), nil
	}

	c := mongodoc.NewProjectConsumer(r.f.Readable)
	filter := bson.M{"workspace": id.String()}

	if includeArchived == nil || !*includeArchived {
		filter = mongox.And(filter, "archived", bson.M{"$ne": true}).(bson.M)
	}

	if keyword != nil && *keyword != "" {
		re := primitive.Regex{Pattern: fmt.Sprintf(".*%s.*", regexp.QuoteMeta(*keyword)), Options: "i"}
		filter = mongox.And(filter, "name", bson.M{"$regex": re}).(bson.M)
	}

	if len(createdBy) > 0 {
		filter = mongox.And(filter, "createdby", bson.M{"$in": createdBy.Strings()}).(bson.M)
	}

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

func (r *Project) CountByWorkspace(ctx context.Context, ws accountsid.WorkspaceID) (int, error) {
	if !r.f.CanRead(ws) {
		return 0, repo.ErrOperationDenied
	}

	count, err := r.client.Count(ctx, bson.M{
		"workspace": ws.String(),
	})
	return int(count), err
}

func (r *Project) CountPublicByWorkspace(ctx context.Context, ws accountsid.WorkspaceID) (int, error) {
	if !r.f.CanRead(ws) {
		return 0, repo.ErrOperationDenied
	}

	count, err := r.client.Count(ctx, bson.M{
		"workspace":  ws.String(),
		"visibility": string(project.VisibilityPublic),
	})
	return int(count), err
}

func (r *Project) Save(ctx context.Context, project *project.Project) error {
	if !r.f.CanWrite(project.Workspace()) {
		return repo.ErrOperationDenied
	}
	doc, id := mongodoc.NewProject(project)
	doc.StarredBy = nil
	if err := r.client.SetOne(ctx, id, doc); err != nil {
		if isAliasConflict(err) {
			return interfaces.ErrProjectAliasAlreadyUsed
		}
		return err
	}
	return nil
}

func isAliasConflict(err error) bool {
	var se mongo.ServerError
	return mongo.IsDuplicateKeyError(err) && errors.As(err, &se) && strings.Contains(se.Error(), projectAliasIndex)
}

// projectSearchSort maps a sort field to the document field it sorts on.
// _starcount is added by the Search pipeline when sorting by stars.
var projectSearchSort = map[interfaces.ProjectSortField]string{
	interfaces.ProjectSortUpdatedAt: "updatedat",
	interfaces.ProjectSortName:      "name",
	interfaces.ProjectSortStarCount: "_starcount",
}

func (r *Project) Search(ctx context.Context, p interfaces.ProjectSearchParam) ([]*project.Project, *interfaces.PageBasedInfo, error) {
	var and []bson.M
	if p.Workspace != nil {
		and = append(and, bson.M{"workspace": p.Workspace.String()})
	}
	if r.f.Readable != nil {
		and = append(and, bson.M{"workspace": bson.M{"$in": r.f.Readable.Strings()}})
	}
	if p.Visibility != nil {
		// Documents saved before visibility existed have none and read as private.
		if *p.Visibility == project.VisibilityPublic {
			and = append(and, bson.M{"visibility": string(project.VisibilityPublic)})
		} else {
			and = append(and, bson.M{"visibility": bson.M{"$ne": string(project.VisibilityPublic)}})
		}
	}
	if !p.IncludeArchived {
		and = append(and, bson.M{"archived": bson.M{"$ne": true}})
	}
	if p.Keyword != "" {
		and = append(and, bson.M{"name": bson.M{"$regex": primitive.Regex{Pattern: regexp.QuoteMeta(p.Keyword), Options: "i"}}})
	}
	if len(p.Topics) > 0 {
		and = append(and, bson.M{"topics": bson.M{"$all": p.Topics}})
	}
	filter := bson.M{}
	if len(and) > 0 {
		filter = bson.M{"$and": and}
	}

	total, err := r.client.Count(ctx, filter)
	if err != nil {
		return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
	}

	field, ok := projectSearchSort[p.SortBy]
	if !ok {
		field = projectSearchSort[interfaces.ProjectSortUpdatedAt]
	}
	dir := -1
	if p.SortAscending {
		dir = 1
	}
	// The star count is computed only for the star sort, so the other sorts
	// follow $match directly and could use an index.
	byStars := field == projectSearchSort[interfaces.ProjectSortStarCount]
	pipeline := []any{bson.M{"$match": filter}}
	if byStars {
		pipeline = append(pipeline, bson.M{"$addFields": bson.M{"_starcount": bson.M{"$size": bson.M{"$ifNull": bson.A{"$starredby", bson.A{}}}}}})
	}
	pipeline = append(pipeline,
		bson.M{"$sort": bson.D{{Key: field, Value: dir}, {Key: "id", Value: dir}}},
		bson.M{"$skip": int64((p.Page - 1) * p.PageSize)},
		bson.M{"$limit": int64(p.PageSize)},
	)
	if byStars {
		pipeline = append(pipeline, bson.M{"$unset": "_starcount"})
	}
	c := mongodoc.NewProjectConsumer(r.f.Readable)
	if err := r.client.Aggregate(ctx, pipeline, c); err != nil {
		return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
	}
	return c.Result, interfaces.NewPageBasedInfo(total, p.Page, p.PageSize), nil
}

func (r *Project) SetStar(ctx context.Context, pid id.ProjectID, uid accountsid.UserID, starred bool) error {
	op := "$addToSet"
	if !starred {
		op = "$pull"
	}
	filter := applyWorkspaceFilter(bson.M{"id": pid.String()}, r.f.Readable)
	res, err := r.client.Client().UpdateOne(ctx, filter, bson.M{op: bson.M{"starredby": uid.String()}})
	if err != nil {
		return rerror.ErrInternalByWithContext(ctx, err)
	}
	if res.MatchedCount == 0 {
		return rerror.ErrNotFound
	}
	return nil
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
	var f []accountsid.WorkspaceID
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
