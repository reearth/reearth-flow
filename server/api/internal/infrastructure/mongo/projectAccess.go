package mongo

import (
	"context"

	"go.mongodb.org/mongo-driver/bson"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearth-flow/api/pkg/projectAccess"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/rerror"
)

var (
	projectAccessIndexes       = []string{"token"}
	projectAccessUniqueIndexes = []string{"id", "project"}
)

type ProjectAccess struct {
	client *mongox.ClientCollection
}

func NewProjectAccess(client *mongox.Client) *ProjectAccess {
	return &ProjectAccess{
		client: client.WithCollection("projectAccess"),
	}
}

func (r *ProjectAccess) Init(ctx context.Context) error {
	return createIndexes(ctx, r.client, projectAccessIndexes, projectAccessUniqueIndexes)
}

func (r *ProjectAccess) FindByProjectID(ctx context.Context, id project.ID) (*projectAccess.ProjectAccess, error) {
	return r.findOne(ctx, bson.M{
		"project": id.String(),
	})
}

func (r *ProjectAccess) FindByToken(ctx context.Context, token string) (*projectAccess.ProjectAccess, error) {
	return r.findOne(ctx, bson.M{
		"token": token,
	})
}

func (r *ProjectAccess) Save(ctx context.Context, projectAccess *projectAccess.ProjectAccess) error {
	doc, id := mongodoc.NewProjectAccess(projectAccess)
	return r.client.SaveOne(ctx, id, doc)
}

func (r *ProjectAccess) findOne(ctx context.Context, filter any) (*projectAccess.ProjectAccess, error) {
	c := mongodoc.NewProjectAccessConsumer()
	if err := r.client.FindOne(ctx, filter, c); err != nil {
		return nil, err
	}
	if len(c.Result) == 0 {
		return nil, rerror.ErrNotFound
	}
	return c.Result[0], nil
}
