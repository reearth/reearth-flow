package mongo

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/usecasex"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo/options"
)

var (
	parameterIndexes       = []string{"project"}
	parameterUniqueIndexes = []string{"id"}
)

type Parameter struct {
	client *mongox.ClientCollection
}

func NewParameter(client *mongox.Client) *Parameter {
	return &Parameter{
		client: client.WithCollection("parameter"),
	}
}

func (r *Parameter) Init(ctx context.Context) error {
	return createIndexes(ctx, r.client, parameterIndexes, parameterUniqueIndexes)
}

func (r *Parameter) FindByID(ctx context.Context, id id.ParameterID) (*parameter.Parameter, error) {
	return r.findOne(ctx, bson.M{
		"id": id.String(),
	})
}

func (r *Parameter) FindByIDs(ctx context.Context, ids id.ParameterIDList) (*parameter.ParameterList, error) {
	if len(ids) == 0 {
		return nil, nil
	}

	res, err := r.find(ctx, bson.M{
		"id": bson.M{
			"$in": ids.Strings(),
		},
	})
	if err != nil {
		return nil, err
	}
	return parameter.NewParameterList(res), nil
}

func (r *Parameter) FindByProject(ctx context.Context, pid id.ProjectID) (*parameter.ParameterList, *usecasex.PageInfo, error) {
	filter := bson.M{"project": pid.String()}

	opt := options.Find()
	opt.SetSort(bson.D{{Key: "index", Value: 1}}) // Sort by index ascending

	res, err := r.find(ctx, filter, opt)
	if err != nil {
		return nil, nil, err
	}

	pl := parameter.NewParameterList(res)
	if pl == nil {
		return nil, nil, nil
	}

	return pl, nil, nil
}

func (r *Parameter) Save(ctx context.Context, param *parameter.Parameter) error {
	doc, id := mongodoc.NewParameter(param)
	return r.client.SaveOne(ctx, id, doc)
}

func (r *Parameter) Remove(ctx context.Context, id id.ParameterID) error {
	return r.client.RemoveOne(ctx, bson.M{"id": id.String()})
}

func (r *Parameter) RemoveAll(ctx context.Context, ids id.ParameterIDList) error {
	if len(ids) == 0 {
		return nil
	}
	return r.client.RemoveAll(ctx, bson.M{
		"id": bson.M{
			"$in": ids.Strings(),
		},
	})
}

func (r *Parameter) RemoveAllByProject(ctx context.Context, pid id.ProjectID) error {
	return r.client.RemoveAll(ctx, bson.M{
		"project": pid.String(),
	})
}

func (r *Parameter) find(ctx context.Context, filter interface{}, opts ...*options.FindOptions) ([]*parameter.Parameter, error) {
	c := mongodoc.NewParameterConsumer()
	if err := r.client.Find(ctx, filter, c, opts...); err != nil {
		return nil, err
	}
	return c.Result, nil
}

func (r *Parameter) findOne(ctx context.Context, filter interface{}) (*parameter.Parameter, error) {
	c := mongodoc.NewParameterConsumer()
	if err := r.client.FindOne(ctx, filter, c); err != nil {
		return nil, err
	}
	return c.Result[0], nil
}
