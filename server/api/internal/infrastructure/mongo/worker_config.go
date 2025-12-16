package mongo

import (
	"context"
	"errors"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
	"github.com/reearth/reearthx/mongox"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
)

type WorkerConfig struct {
	client *mongox.ClientCollection
}

func NewWorkerConfig(client *mongox.Client) repo.WorkerConfig {
	return &WorkerConfig{client: client.WithCollection("worker_config")}
}

func (r *WorkerConfig) FindByID(ctx context.Context, wid id.WorkerConfigID) (*workerconfig.WorkerConfig, error) {
	return r.findOne(ctx, bson.M{"id": wid.String()})
}

func (r *WorkerConfig) FindByIDs(ctx context.Context, ids []id.WorkerConfigID) ([]*workerconfig.WorkerConfig, error) {
	if len(ids) == 0 {
		return nil, nil
	}

	idStrs := make([]string, len(ids))
	for i, wid := range ids {
		idStrs[i] = wid.String()
	}

	filter := bson.M{
		"id": bson.M{
			"$in": idStrs,
		},
	}

	c := mongodoc.NewWorkerConfigConsumer()
	if err := r.client.Find(ctx, filter, c); err != nil {
		return nil, err
	}

	return c.Result, nil
}

func (r *WorkerConfig) FindAll(ctx context.Context) (*workerconfig.WorkerConfig, error) {
	c := mongodoc.NewWorkerConfigConsumer()
	if err := r.client.Find(ctx, bson.M{}, c); err != nil {
		return nil, err
	}
	if len(c.Result) == 0 {
		return nil, nil
	}
	return c.Result[0], nil
}

func (r *WorkerConfig) Save(ctx context.Context, cfg *workerconfig.WorkerConfig) error {
	d, docID := mongodoc.NewWorkerConfig(cfg)
	if d == nil {
		return nil
	}
	return r.client.SaveOne(ctx, docID, d)
}

func (r *WorkerConfig) Remove(ctx context.Context, wid id.WorkerConfigID) error {
	return r.client.RemoveOne(ctx, bson.M{"id": wid.String()})
}

func (r *WorkerConfig) findOne(ctx context.Context, filter interface{}) (*workerconfig.WorkerConfig, error) {
	c := mongodoc.NewWorkerConfigConsumer()
	if err := r.client.FindOne(ctx, filter, c); err != nil {
		if errors.Is(err, mongo.ErrNoDocuments) {
			return nil, nil
		}
		return nil, err
	}
	if len(c.Result) == 0 {
		return nil, nil
	}
	return c.Result[0], nil
}
