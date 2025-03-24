package mongo

import (
	"context"
	"fmt"
	"log"

	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/subscriber/pkg/node"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/usecasex"
	"go.mongodb.org/mongo-driver/bson"
)

type MongoStorage struct {
	client      *mongox.ClientCollection
	transaction usecasex.Transaction
	baseURL     string
	gcsBucket   string
}

func NewMongoStorage(client *mongox.Client, gcsBucket, baseURL string) *MongoStorage {
	transaction := &usecasex.NopTransaction{}

	return &MongoStorage{
		client:      client.WithCollection("nodeExecutions"),
		transaction: transaction,
		baseURL:     baseURL,
		gcsBucket:   gcsBucket,
	}
}

type BSONConsumer struct {
	Result interface{}
}

func (c *BSONConsumer) Consume(raw bson.Raw) error {
	if c.Result == nil {
		return nil
	}

	return bson.Unmarshal(raw, c.Result)
}

func (m *MongoStorage) FindByID(ctx context.Context, id string) (*node.NodeExecution, error) {
	filter := bson.M{
		"id": id,
	}

	c := mongodoc.NewNodeExecutionConsumer()
	if err := m.client.FindOne(ctx, filter, c); err != nil {
		return nil, nil
	}

	if len(c.Result) == 0 {
		return nil, nil
	}

	return c.Result[0], nil
}

func (m *MongoStorage) SaveNodeExecutionToMongo(ctx context.Context, jobID string, nodeExec *node.NodeExecution) error {
	if nodeExec == nil {
		log.Printf("ERROR: Attempted to save nil node execution for jobID=%s", jobID)
		return fmt.Errorf("node execution is nil")
	}

	log.Printf("DEBUG: Saving node execution to MongoDB for jobID=%s, nodeID=%s, status=%s",
		jobID, nodeExec.NodeID, nodeExec.Status)

	existingNode, err := m.FindByID(ctx, nodeExec.ID)
	if err != nil {
		log.Printf("ERROR: Error checking for existing node execution: %v", err)
		return fmt.Errorf("error checking for existing node execution: %w", err)
	}

	nodeDoc := mongodoc.NewNodeExecution(nodeExec)

	if existingNode != nil {
		log.Printf("DEBUG: Node execution record already exists, updating instead of creating duplicate")

		filter := bson.M{
			"jobId":  jobID,
			"nodeId": nodeExec.NodeID,
		}

		update := bson.M{
			"status": string(nodeExec.Status),
		}

		if nodeExec.StartedAt != nil {
			update["startedAt"] = nodeExec.StartedAt
		}

		if nodeExec.CompletedAt != nil {
			update["completedAt"] = nodeExec.CompletedAt
		}

		if err := m.client.UpdateMany(ctx, filter, update); err != nil {
			log.Printf("ERROR: Failed to update node execution: %v", err)
			return fmt.Errorf("failed to update node execution: %w", err)
		}

		log.Printf("DEBUG: Successfully updated existing node execution")
	} else {
		log.Printf("DEBUG: Creating new node execution record")

		if err := m.client.SaveOne(ctx, nodeDoc.ID, nodeDoc); err != nil {
			log.Printf("ERROR: Failed to save node execution: %v", err)
			return fmt.Errorf("failed to save node execution: %w", err)
		}

		log.Printf("DEBUG: Successfully saved new node execution")
	}

	return nil
}
