package mongo_test

import (
	"context"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/mongo"
	"github.com/reearth/reearth-flow/subscriber/pkg/edge"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/mongox/mongotest"
	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/bson"
)

func TestMongoStorage_UpdateEdgeStatusInMongo(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()
	storage := mongo.NewMongoStorage(mongox.NewClientWithDatabase(c), "test-bucket", "https://storage.googleapis.com")

	timeNow := time.Now()
	tests := []struct {
		name      string
		jobID     string
		edgeExec  *edge.EdgeExecution
		expectErr bool
	}{
		{
			name:  "Success",
			jobID: "job-123",
			edgeExec: &edge.EdgeExecution{
				ID:                  "edge-123",
				Status:              edge.StatusCompleted,
				StartedAt:           &timeNow,
				CompletedAt:         &timeNow,
				IntermediateDataURL: "https://storage.googleapis.com/test-bucket/data.json",
			},
			expectErr: false,
		},
		{
			name:  "Job Not Found",
			jobID: "job-456",
			edgeExec: &edge.EdgeExecution{
				ID:     "edge-456",
				Status: edge.StatusFailed,
			},
			expectErr: true,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()

			if !tc.expectErr {
				_, err := c.Collection("jobs").InsertOne(ctx, bson.M{"id": tc.jobID})
				assert.NoError(t, err)
			}

			err := storage.UpdateEdgeStatusInMongo(ctx, tc.jobID, tc.edgeExec)

			if tc.expectErr {
				assert.Error(t, err)
				return
			}

			assert.NoError(t, err)

			var updatedJob bson.M
			err = c.Collection("jobs").FindOne(ctx, bson.M{"id": tc.jobID}).Decode(&updatedJob)
			assert.NoError(t, err)

			assert.Equal(t, tc.edgeExec.ID, updatedJob["edge_execution"].(bson.M)["id"])
			assert.Equal(t, tc.edgeExec.Status, updatedJob["edge_execution"].(bson.M)["status"])
			assert.Equal(t, tc.edgeExec.IntermediateDataURL, updatedJob["edge_execution"].(bson.M)["intermediate_data_url"])

			if tc.edgeExec.StartedAt != nil {
				assert.WithinDuration(t, *tc.edgeExec.StartedAt, updatedJob["edge_execution"].(bson.M)["started_at"].(time.Time), time.Second)
			}
			if tc.edgeExec.CompletedAt != nil {
				assert.WithinDuration(t, *tc.edgeExec.CompletedAt, updatedJob["edge_execution"].(bson.M)["completed_at"].(time.Time), time.Second)
			}
		})
	}
}

func TestMongoStorage_ConstructIntermediateDataURL(t *testing.T) {
	c := mongotest.Connect(t)(t)
	storage := mongo.NewMongoStorage(mongox.NewClientWithDatabase(c), "test-bucket", "https://storage.googleapis.com")

	tests := []struct {
		name        string
		jobID       string
		edgeID      string
		expectedURL string
	}{
		{
			name:        "Valid Job and Edge ID",
			jobID:       "job-789",
			edgeID:      "edge-789",
			expectedURL: "https://storage.googleapis.com/test-bucket/artifacts/job-789/feature-store/edge-789.jsonl",
		},
		{
			name:        "Special Characters in Job and Edge ID",
			jobID:       "job@#$%",
			edgeID:      "edge&*()",
			expectedURL: "https://storage.googleapis.com/test-bucket/artifacts/job@#$%/feature-store/edge&*().jsonl",
		},
		{
			name:        "Empty Job ID",
			jobID:       "",
			edgeID:      "edge-xyz",
			expectedURL: "https://storage.googleapis.com/test-bucket/artifacts//feature-store/edge-xyz.jsonl",
		},
		{
			name:        "Empty Edge ID",
			jobID:       "job-abc",
			edgeID:      "",
			expectedURL: "https://storage.googleapis.com/test-bucket/artifacts/job-abc/feature-store/.jsonl",
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			edgeURL := storage.ConstructIntermediateDataURL(tc.jobID, tc.edgeID)
			assert.Equal(t, tc.expectedURL, edgeURL)
		})
	}
}
