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

func (r *WorkerConfig) FindByWorkspace(ctx context.Context, workspace id.WorkspaceID) (*workerconfig.WorkerConfig, error) {
	return r.findOne(ctx, bson.M{"workspace": workspace.String()})
}

func (r *WorkerConfig) Save(ctx context.Context, cfg *workerconfig.WorkerConfig) error {
	d, id := mongodoc.NewWorkerConfig(cfg)
	if d == nil {
		return nil
	}
	return r.client.SaveOne(ctx, id, d)
}

func (r *WorkerConfig) Remove(ctx context.Context, workspace id.WorkspaceID) error {
	return r.client.RemoveOne(ctx, bson.M{"workspace": workspace.String()})
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
