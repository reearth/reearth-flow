package mongo

import (
	"context"
	"errors"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/batchconfig"
	"github.com/reearth/reearth-flow/api/pkg/id"
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

func (r *WorkerConfig) FindByWorkspace(ctx context.Context, workspace id.WorkspaceID) (*batchconfig.WorkerConfig, error) {
	consumer := mongodoc.NewWorkerConfigConsumer(workspace)
	if err := r.client.Find(ctx, bson.M{"workspace": workspace.String()}, consumer); err != nil {
		if errors.Is(err, mongo.ErrNoDocuments) {
			return nil, nil
		}
		return nil, err
	}
	res := consumer.Result
	if len(res) == 0 {
		return nil, nil
	}
	return res[0], nil
}

func (r *WorkerConfig) Save(ctx context.Context, cfg *batchconfig.WorkerConfig) error {
	d, id := mongodoc.NewWorkerConfig(cfg)
	if d == nil {
		return nil
	}
	return r.client.SaveOne(ctx, id, d)
}

func (r *WorkerConfig) Remove(ctx context.Context, workspace id.WorkspaceID) error {
	return r.client.RemoveOne(ctx, bson.M{"workspace": workspace.String()})
}
