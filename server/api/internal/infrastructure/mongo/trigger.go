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

func (r *Trigger) FindByWorkspace(ctx context.Context, id accountdomain.WorkspaceID, pagination *interfaces.PaginationParam) ([]*trigger.Trigger, *interfaces.PageBasedInfo, error) {
	if !r.f.CanRead(id) {
		return nil, interfaces.NewPageBasedInfo(0, 1, 1), nil
	}

	c := mongodoc.NewTriggerConsumer(r.f.Readable)
	filter := bson.M{"workspaceid": id.String()}

	if pagination != nil && pagination.Page != nil {
		skip := int64((pagination.Page.Page - 1) * pagination.Page.PageSize)
		limit := int64(pagination.Page.PageSize)

		total, err := r.client.Count(ctx, filter)
		if err != nil {
			return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
		}

		opts := options.Find().SetSkip(skip).SetLimit(limit)
		if pagination.Page.OrderBy != nil {
			direction := 1
			if pagination.Page.OrderDir != nil && *pagination.Page.OrderDir == "DESC" {
				direction = -1
			}

			fieldNameMap := map[string]string{
				"description": "description",
				"createdAt":   "createdat",
				"updatedAt":   "updatedat",
				"status":      "status",
				"id":          "id",
			}

			fieldName := *pagination.Page.OrderBy
			if mongoField, ok := fieldNameMap[fieldName]; ok {
				fieldName = mongoField
			}
			opts.SetSort(bson.D{{Key: fieldName, Value: direction}})
		} else {
			opts.SetSort(bson.D{{Key: "updatedat", Value: -1}})
		}

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

func (r *Trigger) Save(ctx context.Context, trigger *trigger.Trigger) error {
	if !r.f.CanWrite(trigger.Workspace()) {
		return repo.ErrOperationDenied
	}
	doc, id := mongodoc.NewTrigger(trigger)
	return r.client.SaveOne(ctx, id, doc)
}

func (r *Trigger) Remove(ctx context.Context, id id.TriggerID) error {
	return r.client.RemoveOne(ctx, r.writeFilter(bson.M{
		"id": id.String(),
	}))
}

func (r *Trigger) writeFilter(filter any) any {
	if r.f.Writable == nil {
		return filter
	}
	return mongox.And(filter, "workspaceid", bson.M{"$in": r.f.Writable.Strings()})
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
