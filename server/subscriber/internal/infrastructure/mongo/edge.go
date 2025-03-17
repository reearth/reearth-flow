package mongo

import (
	"context"
	"errors"
	"fmt"
	"log"
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
	log.Printf("DEBUG: Initializing MongoStorage with gcsBucket=%s, baseURL=%s", gcsBucket, baseURL)
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
		log.Printf("DEBUG: Consume received nil raw BSON")
		return nil
	}

	c.found = true
	log.Printf("DEBUG: Found document in MongoDB")

	if err := raw.Lookup("status").Unmarshal(&c.Status); err != nil {
		log.Printf("ERROR: Failed to unmarshal status: %v", err)
		return fmt.Errorf("failed to unmarshal status: %v", err)
	}
	log.Printf("DEBUG: Unmarshaled job status: %s", c.Status)

	if val, err := raw.LookupErr("edgeExecutions"); err == nil {
		if err := val.Unmarshal(&c.EdgeExecutions); err != nil {
			log.Printf("ERROR: Failed to unmarshal edge executions: %v", err)
			return fmt.Errorf("failed to unmarshal edge executions: %v", err)
		}
		log.Printf("DEBUG: Unmarshaled %d edge executions", len(c.EdgeExecutions))
	} else {
		log.Printf("DEBUG: No edgeExecutions found in document")
	}

	return nil
}

func (m *MongoStorage) UpdateEdgeStatusInMongo(ctx context.Context, jobID string, edgeExec *edge.EdgeExecution) error {
	if edgeExec == nil {
		log.Printf("ERROR: Attempted to update nil edge execution for jobID=%s", jobID)
		return fmt.Errorf("edge execution is nil")
	}

	log.Printf("DEBUG: Updating edge status in MongoDB for jobID=%s, edgeID=%s, status=%s", 
		jobID, edgeExec.ID, edgeExec.Status)
	
	filter := bson.M{
		"id":                jobID,
		"edgeExecutions.id": edgeExec.ID,
	}
	log.Printf("DEBUG: Using filter: %+v", filter)

	edgeDoc := mongodoc.NewEdgeExecution(edgeExec)
	log.Printf("DEBUG: Created edge execution document with status=%s", edgeDoc.Status)

	updateFields := bson.M{
		"edgeExecutions.$.status": edgeDoc.Status,
	}

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

	log.Printf("DEBUG: Attempting to update existing edge execution with fields: %+v", updateFields)
	err := m.client.UpdateMany(ctx, filter, updateFields)
	if err != nil {
		log.Printf("DEBUG: Edge execution not found, will attempt to add it. Error: %v", err)
		
		consumer := &jobStatusConsumer{}
		err := m.client.FindOne(ctx, bson.M{"id": jobID}, consumer)

		if err != nil {
			log.Printf("ERROR: Failed to get job %s: %v", jobID, err)
			return fmt.Errorf("failed to get job: %w", err)
		}

		edgeExecutions := []mongodoc.EdgeExecutionDocument{edgeDoc}
		update := bson.M{"edgeExecutions": edgeExecutions}

		if consumer.Status == string(edge.JobStatusPending) {
			update["status"] = string(edge.JobStatusRunning)
			log.Printf("DEBUG: Job was in Pending state, changing to Running")
		}

		log.Printf("DEBUG: Adding new edge execution to job. Update: %+v", update)
		if err := m.client.SetOne(ctx, jobID, update); err != nil {
			log.Printf("ERROR: Failed to add edge to job %s: %v", jobID, err)
			return fmt.Errorf("failed to add edge to job: %w", err)
		}
		log.Printf("DEBUG: Successfully added edge execution to job")
	} else {
		log.Printf("DEBUG: Successfully updated existing edge execution")
	}

	log.Printf("DEBUG: Checking if job status needs to be updated")
	return m.checkAndUpdateJobStatus(ctx, jobID)
}

func (m *MongoStorage) checkAndUpdateJobStatus(ctx context.Context, jobID string) error {
	log.Printf("DEBUG: Checking job status for jobID=%s", jobID)

	consumer := &jobStatusConsumer{}
	err := m.client.FindOne(ctx, bson.M{"id": jobID}, consumer)
	if err != nil {
		log.Printf("ERROR: Failed to get job %s: %v", jobID, err)
		return fmt.Errorf("failed to get job: %w", err)
	}

	if !consumer.found {
		log.Printf("ERROR: Job not found: %s", jobID)
		return fmt.Errorf("job not found: %s", jobID)
	}

	log.Printf("DEBUG: Current job status: %s with %d edge executions", 
		consumer.Status, len(consumer.EdgeExecutions))

	if consumer.Status == string(edge.JobStatusCompleted) || consumer.Status == string(edge.JobStatusFailed) {
		log.Printf("DEBUG: Job already in terminal state (%s), no status update needed", consumer.Status)
		return nil
	}

	if len(consumer.EdgeExecutions) == 0 {
		log.Printf("DEBUG: No edge executions found, no status update needed")
		return nil
	}

	allCompleted := true
	anyFailed := false

	for i, e := range consumer.EdgeExecutions {
		log.Printf("DEBUG: Edge execution %d/%d: ID=%s, Status=%s", 
			i+1, len(consumer.EdgeExecutions), e.ID, e.Status)
		
		if e.Status != string(edge.StatusCompleted) {
			allCompleted = false
		}
		if e.Status == string(edge.StatusFailed) {
			anyFailed = true
			log.Printf("DEBUG: Found failed edge execution: %s", e.ID)
			break
		}
	}

	now := time.Now()

	if anyFailed {
		log.Printf("DEBUG: One or more edge executions failed, updating job status to Failed")
		update := bson.M{
			"status":      string(edge.JobStatusFailed),
			"completedat": now,
		}

		if err := m.client.SetOne(ctx, jobID, update); err != nil {
			log.Printf("ERROR: Failed to update job status to failed: %v", err)
			return fmt.Errorf("failed to update job status to failed: %w", err)
		}
		log.Printf("DEBUG: Successfully updated job status to Failed with completedAt=%s", 
			now.Format(time.RFC3339))
	} else if allCompleted {
		log.Printf("DEBUG: All edge executions completed, updating job status to Completed")
		update := bson.M{
			"status":      string(edge.JobStatusCompleted),
			"completedat": now,
		}

		if err := m.client.SetOne(ctx, jobID, update); err != nil {
			log.Printf("ERROR: Failed to update job status to completed: %v", err)
			return fmt.Errorf("failed to update job status to completed: %w", err)
		}
		log.Printf("DEBUG: Successfully updated job status to Completed with completedAt=%s", 
			now.Format(time.RFC3339))
	} else {
		log.Printf("DEBUG: Job still in progress, no status update needed")
	}

	return nil
}

func (m *MongoStorage) ConstructIntermediateDataURL(jobID, edgeID string) string {
	log.Printf("DEBUG: Constructing intermediate data URL for jobID=%s, edgeID=%s", jobID, edgeID)
	
	const artifactBasePath = "artifacts"
	const featureStorePath = "feature-store"

	edgeDataPath := path.Join(artifactBasePath, jobID, featureStorePath, edgeID+".jsonl")
	log.Printf("DEBUG: Edge data path: %s", edgeDataPath)

	url, err := getGCSObjectURL(m.gcsBucket, m.baseURL, edgeDataPath)
	if err != nil {
		log.Printf("ERROR: Failed to get GCS object URL: %v", err)
		return ""
	}
	log.Printf("DEBUG: Constructed intermediate data URL: %s", url.String())
	return url.String()
}

func getGCSObjectURL(bucketName, base, objectName string) (*url.URL, error) {
	if bucketName == "" {
		log.Printf("ERROR: Bucket name is empty")
		return nil, errors.New("bucket name is empty")
	}

	var u *url.URL
	var err error

	if base == "" {
		base = fmt.Sprintf("https://storage.googleapis.com/%s", bucketName)
		log.Printf("DEBUG: Using default GCS URL format with bucket %s", bucketName)
	}

	u, err = url.Parse(base)
	if err != nil {
		log.Printf("ERROR: Invalid base URL %s: %v", base, err)
		return nil, errors.New("invalid base URL")
	}

	b := *u
	b.Path = path.Join(b.Path, objectName)
	log.Printf("DEBUG: Final GCS object URL: %s", b.String())
	return &b, nil
}
