package mongo

import (
	"context"
	"fmt"
	"log"
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
	baseURL     string
	gcsBucket   string
	transaction usecasex.Transaction
}

func NewMongoStorage(client *mongox.Client, gcsBucket, baseURL string) *MongoStorage {
	log.Printf("DEBUG: Initializing MongoStorage with gcsBucket=%s, baseURL=%s", gcsBucket, baseURL)

	transaction := &usecasex.NopTransaction{}

	return &MongoStorage{
		client:      client.WithCollection("job"),
		baseURL:     baseURL,
		gcsBucket:   gcsBucket,
		transaction: transaction,
	}
}

func (m *MongoStorage) UpdateEdgeStatusInMongo(ctx context.Context, jobID string, edgeExec *edge.EdgeExecution) error {
	if edgeExec == nil {
		log.Printf("ERROR: Attempted to update nil edge execution for jobID=%s", jobID)
		return fmt.Errorf("edge execution is nil")
	}

	log.Printf("DEBUG: Updating edge status in MongoDB for jobID=%s, edgeID=%s, status=%s",
		jobID, edgeExec.ID, edgeExec.Status)

	tx, err := m.transaction.Begin(ctx)
	if err != nil {
		log.Printf("ERROR: Failed to begin transaction: %v", err)
		return fmt.Errorf("failed to begin transaction: %w", err)
	}

	txCtx := tx.Context()

	defer func() {
		if err := tx.End(txCtx); err != nil {
			log.Printf("ERROR: Transaction end failed: %v", err)
		}
	}()

	if err != nil {
		log.Printf("ERROR: Failed to find job %s: %v", jobID, err)
		return fmt.Errorf("failed to find job: %w", err)
	}

	edgeDoc := mongodoc.NewEdgeExecution(edgeExec)
	log.Printf("DEBUG: Created edge execution document with status=%s", edgeDoc.Status)

	filter := bson.M{
		"id":                jobID,
		"edgeExecutions.id": edgeExec.ID,
	}

	count, err := m.client.Count(txCtx, filter)
	if err != nil {
		log.Printf("ERROR: Failed to count edge executions: %v", err)
		return fmt.Errorf("failed to count edge executions: %w", err)
	}

	if count > 0 {
		log.Printf("DEBUG: Updating existing edge execution %s for job %s", edgeExec.ID, jobID)

		updateFields := bson.M{}

		updateFields["edgeExecutions.$.status"] = edgeDoc.Status

		if edgeDoc.StartedAt != nil {
			updateFields["edgeExecutions.$.startedAt"] = edgeDoc.StartedAt
			log.Printf("DEBUG: Adding startedAt=%s to update", edgeDoc.StartedAt.Format(time.RFC3339))
		}

		if edgeDoc.CompletedAt != nil {
			updateFields["edgeExecutions.$.completedAt"] = edgeDoc.CompletedAt
			log.Printf("DEBUG: Adding completedAt=%s to update", edgeDoc.CompletedAt.Format(time.RFC3339))
		}

		if edgeDoc.FeatureID != nil {
			updateFields["edgeExecutions.$.featureId"] = edgeDoc.FeatureID
			log.Printf("DEBUG: Adding featureId=%s to update", *edgeDoc.FeatureID)
		}

		if edgeDoc.IntermediateDataURL != "" {
			updateFields["edgeExecutions.$.intermediateDataUrl"] = edgeDoc.IntermediateDataURL
			log.Printf("DEBUG: Adding intermediateDataUrl=%s to update", edgeDoc.IntermediateDataURL)
		}

		if err := m.client.UpdateMany(txCtx, filter, updateFields); err != nil {
			log.Printf("ERROR: Failed to update existing edge execution: %v", err)
			return fmt.Errorf("failed to update existing edge execution: %w", err)
		}

		log.Printf("DEBUG: Successfully updated existing edge execution")

	} else {
		log.Printf("DEBUG: Adding new edge execution %s to job %s", edgeExec.ID, jobID)

		edgeExecDoc := bson.M{
			"id":                  edgeDoc.ID,
			"status":              edgeDoc.Status,
			"featureId":           edgeDoc.FeatureID,
			"intermediateDataUrl": edgeDoc.IntermediateDataURL,
		}

		if edgeDoc.StartedAt != nil {
			edgeExecDoc["startedAt"] = edgeDoc.StartedAt
		}

		if edgeDoc.CompletedAt != nil {
			edgeExecDoc["completedAt"] = edgeDoc.CompletedAt
		}

		collection := m.client.Client()
		jobFilter := bson.M{"id": jobID}

		push := bson.M{"$push": bson.M{"edgeExecutions": edgeExecDoc}}

		if _, err := collection.UpdateOne(txCtx, jobFilter, push); err != nil {
			log.Printf("ERROR: Failed to add edge execution to job: %v", err)
			return fmt.Errorf("failed to add edge execution to job: %w", err)
		}

		log.Printf("DEBUG: Successfully added edge execution to job")
	}

	tx.Commit()
	log.Printf("DEBUG: Transaction committed successfully")

	return nil
}

func (m *MongoStorage) ConstructIntermediateDataURL(jobID, edgeID string) string {
	log.Printf("DEBUG: Constructing intermediate data URL for jobID=%s, edgeID=%s", jobID, edgeID)

	const artifactBasePath = "artifacts"
	const featureStorePath = "feature-store"

	edgeDataPath := path.Join(artifactBasePath, jobID, featureStorePath, edgeID+".jsonl")
	log.Printf("DEBUG: Edge data path: %s", edgeDataPath)

	var bucketName = m.gcsBucket
	if bucketName == "" {
		log.Printf("WARN: GCS bucket name is empty, using placeholder name")
		bucketName = "placeholder-bucket"
	}

	baseURL := m.baseURL
	if baseURL == "" {
		baseURL = fmt.Sprintf("https://storage.googleapis.com/%s", bucketName)
		log.Printf("DEBUG: Using default GCS URL format with bucket %s", bucketName)
	}

	u, err := url.Parse(baseURL)
	if err != nil {
		log.Printf("ERROR: Invalid base URL %s: %v", baseURL, err)
		return ""
	}

	b := *u
	b.Path = path.Join(b.Path, edgeDataPath)
	dataURL := b.String()
	log.Printf("DEBUG: Constructed intermediate data URL: %s", dataURL)
	return dataURL
}
