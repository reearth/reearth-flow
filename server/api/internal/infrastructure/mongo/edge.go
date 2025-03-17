package mongo

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/edge"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/rerror"
	"go.mongodb.org/mongo-driver/bson"
)

var (
	edgeExecutionIndexes       = []string{"jobId", "edgeId", "status"}
	edgeExecutionUniqueIndexes = []string{"id"}
)

type EdgeExecution struct {
	client *mongox.ClientCollection
}

func NewEdgeExecution(client *mongox.Client) repo.EdgeExecution {
	return &EdgeExecution{
		client: client.WithCollection("edge_executions"),
	}
}

func (r *EdgeExecution) Init(ctx context.Context) error {
	return createIndexes(ctx, r.client, edgeExecutionIndexes, edgeExecutionUniqueIndexes)
}

func (r *EdgeExecution) FindByJobEdgeID(ctx context.Context, jobID id.JobID, edgeID string) (*edge.EdgeExecution, error) {
	filter := bson.M{
		"jobId":  jobID.String(),
		"edgeId": edgeID,
	}
	return r.findOne(ctx, filter)
}

func (r *EdgeExecution) FindByID(ctx context.Context, id string) (*edge.EdgeExecution, error) {
	return r.findOne(ctx, bson.M{
		"id": id,
	})
}

func (r *EdgeExecution) FindByJobID(ctx context.Context, jobID id.JobID) ([]*edge.EdgeExecution, error) {
	filter := bson.M{
		"jobId": jobID.String(),
	}
	return r.find(ctx, filter)
}

func (r *EdgeExecution) find(ctx context.Context, filter interface{}) ([]*edge.EdgeExecution, error) {
	c := mongodoc.NewEdgeExecutionConsumer()
	if err := r.client.Find(ctx, filter, c); err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, err)
	}
	return c.Result, nil
}

func (r *EdgeExecution) findOne(ctx context.Context, filter any) (*edge.EdgeExecution, error) {
	c := mongodoc.NewEdgeExecutionConsumer()
	if err := r.client.FindOne(ctx, filter, c); err != nil {
		return nil, err
	}
	return c.Result[0], nil
}
