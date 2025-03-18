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
		client:      client.WithCollection("edgeExecutions"),
		transaction: transaction,
		baseURL:     baseURL,
		gcsBucket:   gcsBucket,
	}
}

func (m *MongoStorage) FindEdgeExecution(ctx context.Context, jobID string, edgeID string) (*edge.EdgeExecution, error) {
	filter := bson.M{
		"jobId":  jobID,
		"edgeId": edgeID,
	}

	c := mongodoc.NewEdgeExecutionConsumer()
	if err := m.client.FindOne(ctx, filter, c); err != nil {
		return nil, err
	}

	if len(c.Result) == 0 {
		return nil, nil
	}

	return c.Result[0], nil
}

func (m *MongoStorage) UpdateEdgeStatusInMongo(ctx context.Context, jobID string, edgeExec *edge.EdgeExecution) error {
	if edgeExec == nil {
		return fmt.Errorf("edge execution is nil")
	}

	tx, err := m.transaction.Begin(ctx)
	if err != nil {
		return fmt.Errorf("failed to begin transaction: %w", err)
	}

	txCtx := tx.Context()

	defer func() {
		_ = tx.End(txCtx)
	}()

	existingExec, _ := m.FindEdgeExecution(txCtx, jobID, edgeExec.EdgeID)

	if existingExec != nil {
		update := bson.M{
			"status":    string(edgeExec.Status),
			"updatedAt": time.Now(),
		}

		if edgeExec.FeatureID != nil {
			update["featureId"] = edgeExec.FeatureID
		}

		if edgeExec.StartedAt != nil {
			update["startedAt"] = edgeExec.StartedAt
		}

		if edgeExec.CompletedAt != nil {
			update["completedAt"] = edgeExec.CompletedAt
		}

		if edgeExec.IntermediateDataURL != "" {
			update["intermediateDataUrl"] = edgeExec.IntermediateDataURL
		}

		if err := m.client.SetOne(txCtx, existingExec.ID, update); err != nil {
			return fmt.Errorf("failed to update edge execution: %w", err)
		}
	} else {
		now := time.Now()
		doc := bson.M{
			"id":        edgeExec.ID,
			"edgeId":    edgeExec.EdgeID,
			"jobId":     jobID,
			"status":    string(edgeExec.Status),
			"createdAt": now,
			"updatedAt": now,
		}

		if edgeExec.FeatureID != nil {
			doc["featureId"] = edgeExec.FeatureID
		}

		if edgeExec.StartedAt != nil {
			doc["startedAt"] = edgeExec.StartedAt
		}

		if edgeExec.CompletedAt != nil {
			doc["completedAt"] = edgeExec.CompletedAt
		}

		if edgeExec.IntermediateDataURL != "" {
			doc["intermediateDataUrl"] = edgeExec.IntermediateDataURL
		}

		if err := m.client.SaveOne(txCtx, edgeExec.ID, doc); err != nil {
			return fmt.Errorf("failed to save edge execution: %w", err)
		}
	}

	tx.Commit()

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
	if bucketName == "" && base == "" {
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
