package mongo

import (
	"context"
	"errors"
	"fmt"
	"net/url"
	"path"
	"time"

	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/subscriber/pkg/edge"
	"github.com/reearth/reearthx/mongox"
	"go.mongodb.org/mongo-driver/bson"
)

type MongoStorage struct {
	client    *mongox.ClientCollection
	baseURL   string
	gcsBucket string
}

func NewMongoStorage(client *mongox.Client, gcsBucket, baseURL string) *MongoStorage {
	return &MongoStorage{
		client:    client.WithCollection("job"),
		baseURL:   baseURL,
		gcsBucket: gcsBucket,
	}
}

type jobStatusConsumer struct {
	Status         string
	EdgeExecutions []struct {
		ID     string
		Status string
	}
	found bool
}

func (c *jobStatusConsumer) Consume(raw bson.Raw) error {
	if raw == nil {
		return nil
	}

	c.found = true

	if err := raw.Lookup("status").Unmarshal(&c.Status); err != nil {
		return fmt.Errorf("failed to unmarshal status: %v", err)
	}

	if val, err := raw.LookupErr("edgeExecutions"); err == nil {
		if err := val.Unmarshal(&c.EdgeExecutions); err != nil {
			return fmt.Errorf("failed to unmarshal edge executions: %v", err)
		}
	}

	return nil
}

func (m *MongoStorage) UpdateEdgeStatusInMongo(ctx context.Context, jobID string, edgeExec *edge.EdgeExecution) error {
	filter := bson.M{
		"id":                jobID,
		"edgeExecutions.id": edgeExec.ID,
	}

	edgeDoc := mongodoc.NewEdgeExecution(edgeExec)

	updateFields := bson.M{
		"edgeExecutions.$.status": edgeDoc.Status,
	}

	if edgeDoc.StartedAt != nil {
		updateFields["edgeExecutions.$.startedAt"] = edgeDoc.StartedAt
	}

	if edgeDoc.CompletedAt != nil {
		updateFields["edgeExecutions.$.completedAt"] = edgeDoc.CompletedAt
	}

	if edgeDoc.FeatureID != nil {
		updateFields["edgeExecutions.$.featureId"] = edgeDoc.FeatureID
	}

	if edgeDoc.IntermediateDataURL != "" {
		updateFields["edgeExecutions.$.intermediateDataUrl"] = edgeDoc.IntermediateDataURL
	}

	err := m.client.UpdateMany(ctx, filter, updateFields)
	if err != nil {
		consumer := &jobStatusConsumer{}
		err := m.client.FindOne(ctx, bson.M{"id": jobID}, consumer)

		if err != nil {
			return fmt.Errorf("failed to get job: %w", err)
		}

		edgeExecutions := []mongodoc.EdgeExecutionDocument{edgeDoc}
		update := bson.M{"edgeExecutions": edgeExecutions}

		if consumer.Status == string(edge.JobStatusPending) {
			update["status"] = string(edge.JobStatusRunning)
		}

		if err := m.client.SetOne(ctx, jobID, update); err != nil {
			return fmt.Errorf("failed to add edge to job: %w", err)
		}
	}

	return m.checkAndUpdateJobStatus(ctx, jobID)
}

func (m *MongoStorage) checkAndUpdateJobStatus(ctx context.Context, jobID string) error {

	consumer := &jobStatusConsumer{}
	err := m.client.FindOne(ctx, bson.M{"id": jobID}, consumer)
	if err != nil {
		return fmt.Errorf("failed to get job: %w", err)
	}

	if !consumer.found {
		return fmt.Errorf("job not found: %s", jobID)
	}

	if consumer.Status == string(edge.JobStatusCompleted) || consumer.Status == string(edge.JobStatusFailed) {
		return nil
	}

	if len(consumer.EdgeExecutions) == 0 {
		return nil
	}

	allCompleted := true
	anyFailed := false

	for _, e := range consumer.EdgeExecutions {
		if e.Status != string(edge.StatusCompleted) {
			allCompleted = false
		}
		if e.Status == string(edge.StatusFailed) {
			anyFailed = true
			break
		}
	}

	now := time.Now()

	if anyFailed {
		update := bson.M{
			"status":      string(edge.JobStatusFailed),
			"completedat": now,
		}

		if err := m.client.SetOne(ctx, jobID, update); err != nil {
			return fmt.Errorf("failed to update job status to failed: %w", err)
		}
	} else if allCompleted {
		update := bson.M{
			"status":      string(edge.JobStatusCompleted),
			"completedat": now,
		}

		if err := m.client.SetOne(ctx, jobID, update); err != nil {
			return fmt.Errorf("failed to update job status to completed: %w", err)
		}
	}

	return nil
}

func (m *MongoStorage) ConstructIntermediateDataURL(jobID, edgeID string) string {
	const artifactBasePath = "artifacts"
	const featureStorePath = "feature-store"

	edgeDataPath := path.Join(artifactBasePath, jobID, featureStorePath, edgeID+".jsonl")

	url, err := getGCSObjectURL(m.gcsBucket, m.baseURL, edgeDataPath)
	if err != nil {
		return ""
	}
	return url.String()
}

func getGCSObjectURL(bucketName, base, objectName string) (*url.URL, error) {
	if bucketName == "" {
		return nil, errors.New("bucket name is empty")
	}

	var u *url.URL
	var err error

	if base == "" {
		base = fmt.Sprintf("https://storage.googleapis.com/%s", bucketName)
	}

	u, err = url.Parse(base)
	if err != nil {
		return nil, errors.New("invalid base URL")
	}

	b := *u
	b.Path = path.Join(b.Path, objectName)
	return &b, nil
}
