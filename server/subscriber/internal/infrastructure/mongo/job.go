package mongo

import (
	"context"
	"fmt"

	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"

	"github.com/reearth/reearth-flow/subscriber/pkg/job"
)

type JobStorageMongo struct {
	client     *mongo.Client
	database   string
	collection string
}

func NewJobStorageMongo(client *mongo.Client, database, collection string) *JobStorageMongo {
	return &JobStorageMongo{
		client:     client,
		database:   database,
		collection: collection,
	}
}

func (m *JobStorageMongo) SaveToMongo(ctx context.Context, jobID string, jobRecord *job.Job) error {
	coll := m.client.Database(m.database).Collection(m.collection)

	filter := bson.M{"id": jobID}
	update := bson.M{
		"$set": bson.M{
			"id":         jobRecord.ID,
			"workflowId": jobRecord.WorkflowID,
			"status":     jobRecord.Status,
			"updatedAt":  jobRecord.UpdatedAt,
		},
	}

	if jobRecord.Message != nil {
		update["$set"].(bson.M)["message"] = *jobRecord.Message
	}

	if len(jobRecord.FailedNodes) > 0 {
		update["$set"].(bson.M)["failedNodes"] = jobRecord.FailedNodes
	}

	if jobRecord.StartedAt != nil {
		update["$set"].(bson.M)["startedAt"] = *jobRecord.StartedAt
	}

	if jobRecord.CompletedAt != nil {
		update["$set"].(bson.M)["completedAt"] = *jobRecord.CompletedAt
	}

	opts := options.Update().SetUpsert(true)
	_, err := coll.UpdateOne(ctx, filter, update, opts)
	if err != nil {
		return fmt.Errorf("failed to save job to MongoDB: %w", err)
	}

	return nil
}
