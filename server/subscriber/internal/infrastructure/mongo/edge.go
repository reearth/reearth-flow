package mongo

import (
	"context"
	"errors"
	"fmt"
	"log"
	"net/url"
	"path"
	"time"

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
		client:      client.WithCollection("edge_executions"),
		transaction: transaction,
		baseURL:     baseURL,
		gcsBucket:   gcsBucket,
	}
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
		if err := tx.End(txCtx); err != nil {
			log.Printf("ERROR: Transaction end failed: %v", err)
		}
	}()

	intermediateDataURL := m.ConstructIntermediateDataURL(jobID, edgeExec.ID)

	edgeDoc := bson.M{
		"id":        edgeExec.ID,
		"edgeId":    edgeExec.EdgeID,
		"jobId":     jobID,
		"status":    string(edgeExec.Status),
		"createdAt": time.Now(),
	}

	if edgeExec.StartedAt != nil {
		edgeDoc["startedAt"] = edgeExec.StartedAt
	}

	if edgeExec.CompletedAt != nil {
		edgeDoc["completedAt"] = edgeExec.CompletedAt
	}

	if edgeExec.FeatureID != nil {
		edgeDoc["featureId"] = edgeExec.FeatureID
	}

	if intermediateDataURL != "" {
		edgeDoc["intermediateDataUrl"] = intermediateDataURL
	}

	compositeID := fmt.Sprintf("%s:%s", jobID, edgeExec.ID)

	err = m.client.SaveOne(txCtx, compositeID, edgeDoc)
	if err != nil {
		return fmt.Errorf("failed to save edge execution: %w", err)
	}

	tx.Commit()

	tx.Commit()
	log.Printf("DEBUG: Transaction committed successfully")

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
