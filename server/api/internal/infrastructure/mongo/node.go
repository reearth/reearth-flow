package mongo

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/rerror"
	"go.mongodb.org/mongo-driver/bson"
)

var (
	nodeExecutionIndexes       = []string{"jobId", "status"}
	nodeExecutionUniqueIndexes = []string{"id"}
)

type NodeExecution struct {
	client *mongox.ClientCollection
}

func NewNodeExecution(client *mongox.Client) repo.NodeExecution {
	return &NodeExecution{
		client: client.WithCollection("nodeExecutions"),
	}
}

func (r *NodeExecution) Init(ctx context.Context) error {
	return createIndexes(ctx, r.client, nodeExecutionIndexes, nodeExecutionUniqueIndexes)
}

func (r *NodeExecution) FindByJobNodeID(ctx context.Context, jobID id.JobID, nodeID string) (*graph.NodeExecution, error) {
	filter := bson.M{
		"jobId":  jobID.String(),
		"nodeId": nodeID,
	}
	return r.findOne(ctx, filter)
}

func (r *NodeExecution) FindByID(ctx context.Context, id string) (*graph.NodeExecution, error) {
	return r.findOne(ctx, bson.M{
		"id": id,
	})
}

func (r *NodeExecution) FindByJobID(ctx context.Context, jobID id.JobID) ([]*graph.NodeExecution, error) {
	filter := bson.M{
		"jobId": jobID.String(),
	}
	return r.find(ctx, filter)
}

func (r *NodeExecution) find(ctx context.Context, filter interface{}) ([]*graph.NodeExecution, error) {
	c := mongodoc.NewNodeExecutionConsumer()
	if err := r.client.Find(ctx, filter, c); err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, err)
	}
	return c.Result, nil
}

func (r *NodeExecution) findOne(ctx context.Context, filter any) (*graph.NodeExecution, error) {
	c := mongodoc.NewNodeExecutionConsumer()
	if err := r.client.FindOne(ctx, filter, c); err != nil {
		return nil, err
	}
	return c.Result[0], nil
}
