package mongo

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo/options"
)

var (
	triggerIndexes       = []string{"workspaceid", "deploymentid"}
	triggerUniqueIndexes = []string{"id"}
)

type Trigger struct {
	client *mongox.ClientCollection
	f      repo.WorkspaceFilter
}

func NewTrigger(client *mongox.Client) *Trigger {
	return &Trigger{client: client.WithCollection("trigger")}
}

func (r *Trigger) Init(ctx context.Context) error {
	return createIndexes(ctx, r.client, triggerIndexes, triggerUniqueIndexes)
}

func (r *Trigger) Filtered(f repo.WorkspaceFilter) repo.Trigger {
	return &Trigger{
		client: r.client,
		f:      r.f.Merge(f),
	}
}

func (r *Trigger) FindByID(ctx context.Context, id id.TriggerID) (*trigger.Trigger, error) {
	return r.findOne(ctx, bson.M{
		"id": id.String(),
	}, true)
}

func (r *Trigger) FindByIDs(ctx context.Context, ids id.TriggerIDList) ([]*trigger.Trigger, error) {
	filter := bson.M{
		"id": bson.M{
			"$in": ids.Strings(),
		},
	}
	res, err := r.find(ctx, filter)
	if err != nil {
		return nil, err
	}
	return filterTriggers(ids, res), nil
}

func (r *Trigger) FindByWorkspace(ctx context.Context, workspace accountdomain.WorkspaceID, pagination *usecasex.Pagination) ([]*trigger.Trigger, *usecasex.PageInfo, error) {
	if !r.f.CanRead(workspace) {
		return nil, usecasex.EmptyPageInfo(), nil
	}
	return r.paginate(ctx, bson.M{
		"workspaceid": workspace.String(),
	}, pagination)
}

func (r *Trigger) Save(ctx context.Context, trigger *trigger.Trigger) error {
	if !r.f.CanWrite(trigger.Workspace()) {
		return repo.ErrOperationDenied
	}
	doc, id := mongodoc.NewTrigger(trigger)
	return r.client.SaveOne(ctx, id, doc)
}

func (r *Trigger) Remove(ctx context.Context, id id.TriggerID) error {
	return r.client.RemoveOne(ctx, bson.M{"id": id.String()})
}

func (r *Trigger) find(ctx context.Context, filter interface{}) ([]*trigger.Trigger, error) {
	c := mongodoc.NewTriggerConsumer(r.f.Readable)
	if err := r.client.Find(ctx, filter, c); err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, err)
	}
	return c.Result, nil
}

func (r *Trigger) findOne(ctx context.Context, filter any, filterByWorkspaces bool) (*trigger.Trigger, error) {
	var f []accountdomain.WorkspaceID
	if filterByWorkspaces {
		f = r.f.Readable
	}
	c := mongodoc.NewTriggerConsumer(f)
	if err := r.client.FindOne(ctx, filter, c); err != nil {
		return nil, err
	}
	return c.Result[0], nil
}

func (r *Trigger) paginate(ctx context.Context, filter bson.M, pagination *usecasex.Pagination) ([]*trigger.Trigger, *usecasex.PageInfo, error) {
	c := mongodoc.NewTriggerConsumer(r.f.Readable)

	if pagination != nil && pagination.Offset != nil {
		// Page-based pagination
		skip := pagination.Offset.Offset
		limit := pagination.Offset.Limit

		// Get total count for page info
		total, err := r.client.Count(ctx, filter)
		if err != nil {
			return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
		}

		// Execute find with skip and limit
		opts := options.Find().
			SetSkip(skip).
			SetLimit(limit)

		if err := r.client.Find(ctx, filter, c, opts); err != nil {
			return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
		}

		// Create page-based info
		currentPage := int(skip/limit) + 1
		pageInfo := interfaces.NewPageBasedInfo(total, currentPage, int(limit))

		return c.Result, pageInfo.ToPageInfo(), nil
	}

	// Cursor-based pagination
	pageInfo, err := r.client.Paginate(ctx, filter, nil, pagination, c)
	if err != nil {
		return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
	}
	return c.Result, pageInfo, nil
}

func filterTriggers(ids []id.TriggerID, rows []*trigger.Trigger) []*trigger.Trigger {
	res := make([]*trigger.Trigger, 0, len(ids))
	for _, id := range ids {
		var r2 *trigger.Trigger
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
