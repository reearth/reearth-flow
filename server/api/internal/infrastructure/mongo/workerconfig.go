package mongo

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/rerror"
	"go.mongodb.org/mongo-driver/bson"
)

type WorkerConfig struct {
	client *mongox.ClientCollection
}

func NewWorkerConfig(client *mongox.Client) repo.WorkerConfig {
	return &WorkerConfig{
		client: client.WithCollection("workerconfigs"),
	}
}

func (r *WorkerConfig) FindByWorkspace(ctx context.Context, wsID workerconfig.WorkspaceID) (*workerconfig.WorkerConfig, error) {
	filter := bson.M{
		"workspace_id": wsID.String(),
	}

	dst := &mongodoc.WorkerConfigDocument{}
	if err := r.client.FindOne(ctx, filter, dst); err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, err)
	}

	return dst.Model()
}

func (r *WorkerConfig) Save(ctx context.Context, wc *workerconfig.WorkerConfig) error {
	if wc == nil {
		return nil
	}

	doc := mongodoc.NewWorkerConfig(wc)
	filter := bson.M{
		"_id": doc.ID,
	}

	if err := r.client.SaveOne(ctx, filter, doc); err != nil {
		return rerror.ErrInternalByWithContext(ctx, err)
	}

	return nil
}

func (r *WorkerConfig) Delete(ctx context.Context, id workerconfig.ID) error {
	filter := bson.M{
		"_id": id.String(),
	}

	if err := r.client.RemoveOne(ctx, filter); err != nil {
		return rerror.ErrInternalByWithContext(ctx, err)
	}

	return nil
}

func (r *WorkerConfig) Init(ctx context.Context) error {
	return createIndexes(
		ctx,
		r.client,
		[]string{"workspace_id"},
		[]string{"workspace_id"}, // unique index on workspace_id
	)
}
