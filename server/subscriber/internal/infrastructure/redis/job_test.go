package redis

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
	domainJob "github.com/reearth/reearth-flow/subscriber/pkg/job"
)

func strPtr(s string) *string { return &s }
func u64Ptr(v uint64) *uint64 { return &v }

func TestRedisStorage_SaveToRedis(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	event := &domainJob.JobCompleteEvent{
		WorkflowID: "wf-123",
		JobID:      "job-123",
		Result:     "failed",
		Timestamp:  time.Date(2025, 1, 11, 9, 12, 54, 487779000, time.UTC),
		FailedNodes: []diagnostic.WireDiagnostic{
			{
				Code:                 "internal.invariant_violation",
				Category:             "internal",
				Severity:             "fatal",
				EffectiveDisposition: strPtr("fatal"),
				NodeID:               strPtr("node-1"),
				ActionType:           strPtr("Feature Filter"),
				Message:              "boom",
			},
		},
		AggregatedDiagnostics: []diagnostic.WireDiagnostic{
			{
				Code:                 "gltf.zero_face_solid",
				Category:             "gltf",
				Severity:             "warn",
				EffectiveDisposition: strPtr("warn_drop"),
				NodeID:               strPtr("node-4"),
				ActionType:           strPtr("Gltf Writer"),
				FeatureID:            strPtr("33333333-3333-3333-3333-333333333333"),
				Message:              "solid has zero faces and was dropped",
				Help:                 strPtr("fix your geometry"),
				Aggregated: &diagnostic.WireAggregateInfo{
					Count:            5,
					SampleFeatureIds: []string{"33333333-3333-3333-3333-333333333333"},
				},
			},
		},
		DroppedEventCount: u64Ptr(2),
	}

	expectedKey := "job_complete:job-123"
	expectedVal := []byte(`{"timestamp":"2025-01-11T09:12:54.487779Z","droppedEventCount":2,"workflowId":"wf-123","jobId":"job-123","result":"failed","failedNodes":[{"effectiveDisposition":"fatal","nodeId":"node-1","actionType":"Feature Filter","code":"internal.invariant_violation","category":"internal","severity":"fatal","message":"boom"}],"aggregatedDiagnostics":[{"aggregated":{"sampleFeatureIds":["33333333-3333-3333-3333-333333333333"],"count":5},"effectiveDisposition":"warn_drop","nodeId":"node-4","actionType":"Gltf Writer","featureId":"33333333-3333-3333-3333-333333333333","help":"fix your geometry","code":"gltf.zero_face_solid","category":"gltf","severity":"warn","message":"solid has zero faces and was dropped"}]}`)

	mClient.
		On("Set", mock.Anything, expectedKey, expectedVal, 24*time.Hour).
		Return(nil)

	err := rStorage.SaveToRedis(ctx, event)
	assert.NoError(t, err)
	mClient.AssertExpectations(t)
}

func TestRedisStorage_SaveToRedis_LegacyEventWithoutDiagnostics(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	event := &domainJob.JobCompleteEvent{
		WorkflowID: "wf-123",
		JobID:      "job-123",
		Result:     "success",
		Timestamp:  time.Date(2025, 1, 11, 9, 12, 54, 487779000, time.UTC),
	}

	expectedKey := "job_complete:job-123"
	expectedVal := []byte(`{"timestamp":"2025-01-11T09:12:54.487779Z","workflowId":"wf-123","jobId":"job-123","result":"success"}`)

	mClient.
		On("Set", mock.Anything, expectedKey, expectedVal, 24*time.Hour).
		Return(nil)

	err := rStorage.SaveToRedis(ctx, event)
	assert.NoError(t, err)
	mClient.AssertExpectations(t)
}

func TestRedisStorage_SaveToRedis_Error(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	event := &domainJob.JobCompleteEvent{
		WorkflowID: "wf-123",
		JobID:      "job-123",
		Result:     "failed",
		Timestamp:  time.Now(),
	}

	mClient.
		On("Set", mock.Anything, mock.Anything, mock.Anything, 24*time.Hour).
		Return(errors.New("redis set error"))

	err := rStorage.SaveToRedis(ctx, event)
	assert.Error(t, err)
}
