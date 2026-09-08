package redis

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
)

func TestRedisStorage_SaveDiagnosticToRedis_WithNodeID(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	nodeID := "subgraph-a.node-4"
	actionType := "Gltf Writer"
	featureID := "33333333-3333-3333-3333-333333333333"
	help := "Check the input geometry for degenerate solids before exporting to glTF."
	effectiveDisposition := "warn_drop"

	event := &diagnostic.DiagnosticEvent{
		Schema:     diagnostic.DiagnosticSchemaV1,
		WorkflowID: "11111111-1111-1111-1111-111111111111",
		JobID:      "22222222-2222-2222-2222-222222222222",
		Timestamp:  time.Date(2026, 7, 16, 9, 31, 10, 0, time.UTC),
		WireDiagnostic: diagnostic.WireDiagnostic{
			Code:                 "gltf.zero_face_solid",
			Category:             "gltf",
			Severity:             "warn",
			EffectiveDisposition: &effectiveDisposition,
			NodeID:               &nodeID,
			ActionType:           &actionType,
			FeatureID:            &featureID,
			Message:              "solid has zero faces and was dropped",
			Help:                 &help,
		},
	}

	expectedVal := `{"timestamp":"2026-07-16T09:31:10Z","workflowId":"11111111-1111-1111-1111-111111111111","jobId":"22222222-2222-2222-2222-222222222222","schema":"diagnostic.v1","effectiveDisposition":"warn_drop","nodeId":"subgraph-a.node-4","actionType":"Gltf Writer","featureId":"33333333-3333-3333-3333-333333333333","help":"Check the input geometry for degenerate solids before exporting to glTF.","code":"gltf.zero_face_solid","category":"gltf","severity":"warn","message":"solid has zero faces and was dropped"}`

	nodeKey := "diagnostics:22222222-2222-2222-2222-222222222222:subgraph-a.node-4"
	jobKey := "diagnostics:22222222-2222-2222-2222-222222222222"

	mClient.On("LPush", mock.Anything, nodeKey, []interface{}{expectedVal}).Return(nil)
	mClient.On("Expire", mock.Anything, nodeKey, 24*time.Hour).Return(nil)
	mClient.On("LPush", mock.Anything, jobKey, []interface{}{expectedVal}).Return(nil)
	mClient.On("Expire", mock.Anything, jobKey, 24*time.Hour).Return(nil)

	err := rStorage.SaveDiagnosticToRedis(ctx, event)
	assert.NoError(t, err)
	mClient.AssertExpectations(t)
}

func TestRedisStorage_SaveDiagnosticToRedis_WithoutNodeID_FallsBackToJobBucket(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	event := &diagnostic.DiagnosticEvent{
		Schema:     diagnostic.DiagnosticSchemaV1,
		WorkflowID: "wf-123",
		JobID:      "job-456",
		Timestamp:  time.Date(2026, 7, 16, 9, 31, 10, 0, time.UTC),
		WireDiagnostic: diagnostic.WireDiagnostic{
			Code:     "internal.unclassified",
			Category: "internal",
			Severity: "warn",
			Message:  "job-level diagnostic without a nodeId",
		},
	}

	nodeKey := "diagnostics:job-456:_job"
	jobKey := "diagnostics:job-456"

	mClient.On("LPush", mock.Anything, nodeKey, mock.Anything).Return(nil)
	mClient.On("Expire", mock.Anything, nodeKey, 24*time.Hour).Return(nil)
	mClient.On("LPush", mock.Anything, jobKey, mock.Anything).Return(nil)
	mClient.On("Expire", mock.Anything, jobKey, 24*time.Hour).Return(nil)

	err := rStorage.SaveDiagnosticToRedis(ctx, event)
	assert.NoError(t, err)
	mClient.AssertExpectations(t)
}

func TestRedisStorage_SaveDiagnosticToRedis_EmptyNodeID_FallsBackToJobBucket(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	emptyNodeID := ""
	event := &diagnostic.DiagnosticEvent{
		Schema:     diagnostic.DiagnosticSchemaV1,
		WorkflowID: "wf-123",
		JobID:      "job-456",
		Timestamp:  time.Now(),
		WireDiagnostic: diagnostic.WireDiagnostic{
			Code:     "internal.unclassified",
			Category: "internal",
			Severity: "warn",
			NodeID:   &emptyNodeID,
			Message:  "explicit empty nodeId also falls back",
		},
	}

	nodeKey := "diagnostics:job-456:_job"

	mClient.On("LPush", mock.Anything, nodeKey, mock.Anything).Return(nil)
	mClient.On("Expire", mock.Anything, nodeKey, 24*time.Hour).Return(nil)
	mClient.On("LPush", mock.Anything, "diagnostics:job-456", mock.Anything).Return(nil)
	mClient.On("Expire", mock.Anything, "diagnostics:job-456", 24*time.Hour).Return(nil)

	err := rStorage.SaveDiagnosticToRedis(ctx, event)
	assert.NoError(t, err)
	mClient.AssertExpectations(t)
}

func TestRedisStorage_SaveDiagnosticToRedis_NilEvent(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	err := rStorage.SaveDiagnosticToRedis(ctx, nil)
	assert.Error(t, err)
	mClient.AssertNotCalled(t, "LPush")
}

func TestRedisStorage_SaveDiagnosticToRedis_LPushError(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	event := &diagnostic.DiagnosticEvent{
		Schema:     diagnostic.DiagnosticSchemaV1,
		WorkflowID: "wf-123",
		JobID:      "job-456",
		Timestamp:  time.Now(),
		WireDiagnostic: diagnostic.WireDiagnostic{
			Code:     "gltf.zero_face_solid",
			Category: "gltf",
			Severity: "warn",
			Message:  "boom",
		},
	}

	mClient.On("LPush", mock.Anything, mock.Anything, mock.Anything).Return(errors.New("redis lpush error"))

	err := rStorage.SaveDiagnosticToRedis(ctx, event)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to push diagnostic event to Redis list")
	mClient.AssertNotCalled(t, "Expire", mock.Anything, mock.Anything, mock.Anything)
}

func TestRedisStorage_SaveDiagnosticToRedis_ExpireErrorDoesNotFailWrite(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	event := &diagnostic.DiagnosticEvent{
		Schema:     diagnostic.DiagnosticSchemaV1,
		WorkflowID: "wf-123",
		JobID:      "job-456",
		Timestamp:  time.Now(),
		WireDiagnostic: diagnostic.WireDiagnostic{
			Code:     "gltf.zero_face_solid",
			Category: "gltf",
			Severity: "warn",
			Message:  "boom",
		},
	}

	mClient.On("LPush", mock.Anything, mock.Anything, mock.Anything).Return(nil)
	mClient.On("Expire", mock.Anything, mock.Anything, 24*time.Hour).Return(errors.New("expire failed"))

	err := rStorage.SaveDiagnosticToRedis(ctx, event)
	assert.NoError(t, err)
	mClient.AssertExpectations(t)
}
