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
	"github.com/reearth/reearthx/rerror"
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

	var doc bson.M
	err := m.client.FindOne(ctx, filter, &BSONConsumer{Result: &doc})
	if err != nil {
		if errors.Is(err, rerror.ErrNotFound) {
			return nil, nil
		}
		return nil, err
	}

	if doc == nil {
		return nil, nil
	}

	edgeExec := &edge.EdgeExecution{
		ID:     doc["id"].(string),
		EdgeID: doc["edgeId"].(string),
		Status: edge.Status(doc["status"].(string)),
	}

	if featureID, ok := doc["featureId"]; ok && featureID != nil {
		featureIDStr := featureID.(string)
		edgeExec.FeatureID = &featureIDStr
	}

	if startedAt, ok := doc["startedAt"]; ok && startedAt != nil {
		startedTime := startedAt.(time.Time)
		edgeExec.StartedAt = &startedTime
	}

	if completedAt, ok := doc["completedAt"]; ok && completedAt != nil {
		completedTime := completedAt.(time.Time)
		edgeExec.CompletedAt = &completedTime
	}

	if url, ok := doc["intermediateDataUrl"]; ok && url != nil {
		edgeExec.IntermediateDataURL = url.(string)
	}

	return edgeExec, nil
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

	intermediateDataURL := m.ConstructIntermediateDataURL(jobID, edgeExec.EdgeID)
	if intermediateDataURL != "" {
		edgeExec.IntermediateDataURL = intermediateDataURL
	}

	doc := bson.M{
		"id":        edgeExec.ID,
		"edgeId":    edgeExec.EdgeID,
		"jobId":     jobID,
		"status":    string(edgeExec.Status),
		"updatedAt": time.Now(),
	}

	if edgeExec.FeatureID != nil {
		doc["featureId"] = *edgeExec.FeatureID
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

	filter := bson.M{
		"jobId":  jobID,
		"edgeId": edgeExec.EdgeID,
	}

	var existing bson.M
	err = m.client.FindOne(txCtx, filter, &BSONConsumer{Result: &existing})

	if err == nil && existing != nil {
		if createdAt, ok := existing["createdAt"]; ok && createdAt != nil {
			doc["createdAt"] = createdAt
		}

		err = m.client.SetOne(txCtx, edgeExec.ID, doc)
		if err != nil {
			return fmt.Errorf("failed to update edge execution: %w", err)
		}
	} else {
		doc["createdAt"] = time.Now()

		err = m.client.SaveOne(txCtx, edgeExec.ID, doc)
		if err != nil {
			return fmt.Errorf("failed to save edge execution: %w", err)
		}
	}

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

type BSONConsumer struct {
	Result interface{}
}

func (c *BSONConsumer) Consume(raw bson.Raw) error {
	if raw == nil {
		return nil
	}
	return bson.Unmarshal(raw, c.Result)
}
