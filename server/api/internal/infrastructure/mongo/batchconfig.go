package mongo

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/batchconfig"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/rerror"
	"go.mongodb.org/mongo-driver/bson"
)

var (
	batchConfigIndexes       = []string{"workspaceid"}
	batchConfigUniqueIndexes = []string{"id"}
)

type BatchConfigRepo struct {
	client *mongox.ClientCollection
}

func NewBatchConfig(client *mongox.Client) repo.BatchConfig {
	return &BatchConfigRepo{
		client: client.WithCollection("batchconfig"),
	}
}

func (r *BatchConfigRepo) Init(ctx context.Context) error {
	return createIndexes(ctx, r.client, batchConfigIndexes, batchConfigUniqueIndexes)
}

func (r *BatchConfigRepo) FindByID(ctx context.Context, id batchconfig.ID) (*batchconfig.BatchConfig, error) {
	return r.findOne(ctx, bson.M{
		"id": id.String(),
	})
}

func (r *BatchConfigRepo) FindByWorkspaceID(ctx context.Context, workspaceID batchconfig.WorkspaceID) (*batchconfig.BatchConfig, error) {
	return r.findOne(ctx, bson.M{
		"workspaceid": workspaceID.String(),
	})
}

func (r *BatchConfigRepo) Save(ctx context.Context, config *batchconfig.BatchConfig) error {
	if config == nil {
		return nil
	}

	doc, err := mongodoc.NewBatchConfig(config)
	if err != nil {
		return err
	}

	return r.client.SaveOne(ctx, config.ID().String(), doc)
}

func (r *BatchConfigRepo) Remove(ctx context.Context, id batchconfig.ID) error {
	return r.client.RemoveOne(ctx, bson.M{
		"id": id.String(),
	})
}

func (r *BatchConfigRepo) RemoveByWorkspaceID(ctx context.Context, workspaceID batchconfig.WorkspaceID) error {
	return r.client.RemoveOne(ctx, bson.M{
		"workspaceid": workspaceID.String(),
	})
}

// Helper methods

func (r *BatchConfigRepo) findOne(ctx context.Context, filter interface{}) (*batchconfig.BatchConfig, error) {
	var doc mongodoc.BatchConfigDocument
	if err := r.client.FindOne(ctx, filter, &doc); err != nil {
		if err == rerror.ErrNotFound {
			return nil, nil
		}
		return nil, rerror.ErrInternalByWithContext(ctx, err)
	}
	return doc.Model()
}
