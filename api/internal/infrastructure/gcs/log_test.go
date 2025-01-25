package gcs

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

type mockGCSClient struct {
	mock.Mock
}

func (m *mockGCSClient) Bucket(name string) GCSBucket {
	args := m.Called(name)
	return args.Get(0).(GCSBucket)
}

type mockGCSBucket struct {
	mock.Mock
}

func (m *mockGCSBucket) ListObjects(ctx context.Context, prefix string) ([]string, error) {
	args := m.Called(ctx, prefix)
	return args.Get(0).([]string), args.Error(1)
}

func (m *mockGCSBucket) ReadObject(ctx context.Context, objectName string) ([]byte, error) {
	args := m.Called(ctx, objectName)
	return args.Get(0).([]byte), args.Error(1)
}

func TestNewGCSLog(t *testing.T) {
	_, err := NewGCSLog(nil, "bucket")
	assert.Error(t, err, "should fail if client is nil")

	client := new(mockGCSClient)
	_, err = NewGCSLog(client, "")
	assert.Error(t, err, "should fail if bucketName is empty")

	g, err := NewGCSLog(client, "bucket")
	assert.NoError(t, err, "should succeed with valid input")
	assert.NotNil(t, g, "returned gcsLog should not be nil")
}

func TestGCSLog_GetLogs(t *testing.T) {
	ctx := context.Background()

	// Mock Setup
	mClient := new(mockGCSClient)
	mBucket := new(mockGCSBucket)

	const bucketName = "test-bucket"
	g, err := NewGCSLog(mClient, bucketName)
	assert.NoError(t, err)
	assert.NotNil(t, g)

	// When Bucket(...) is called, return mock bucket
	mClient.
		On("Bucket", bucketName).
		Return(mBucket).
		Once()

	// Test parameters
	since := time.Date(2025, 1, 1, 0, 0, 0, 0, time.UTC)
	until := time.Date(2025, 1, 1, 23, 59, 0, 0, time.UTC)
	wfID := id.NewWorkflowID()
	jbID := id.NewJobID()
	prefix := fmt.Sprintf(
		"artifacts/logs/%04d/%02d/%02d/%s/%s/",
		2025, 1, 1,
		wfID.String(),
		jbID.String(),
	)

	// Mock data to be returned as logs
	logEntry := LogEntry{
		WorkflowID: wfID.String(),
		JobID:      jbID.String(),
		NodeID:     nil,
		Timestamp:  time.Date(2025, 1, 1, 10, 0, 0, 0, time.UTC),
		LogLevel:   log.LevelInfo,
		Message:    "hello from test",
	}
	logJSON, _ := json.Marshal(logEntry)

	// ListObjects returns one object
	objectName := prefix + "2025-01-01T10:00:00Z.json"
	mBucket.
		On("ListObjects", mock.Anything, prefix).
		Return([]string{objectName}, nil).
		Once()

	// ReadObject returns logJSON
	mBucket.
		On("ReadObject", mock.Anything, objectName).
		Return(logJSON, nil).
		Once()

	results, err := g.GetLogs(ctx, since, until, wfID, jbID)
	assert.NoError(t, err)
	assert.Len(t, results, 1, "should return exactly one log within range")

	// Verify the result
	assert.Equal(t, wfID, results[0].WorkflowID())
	assert.Equal(t, jbID, results[0].JobID())
	assert.Nil(t, results[0].NodeID())
	assert.Equal(t, log.LevelInfo, results[0].Level())
	assert.Equal(t, "hello from test", results[0].Message())

	mClient.AssertExpectations(t)
	mBucket.AssertExpectations(t)
}

func TestGCSLog_GetLogs_ListError(t *testing.T) {
	ctx := context.Background()

	mClient := new(mockGCSClient)
	mBucket := new(mockGCSBucket)

	g, _ := NewGCSLog(mClient, "test-bucket")

	mClient.
		On("Bucket", "test-bucket").
		Return(mBucket).
		Once()

	// Force ListObjects to return an error
	mBucket.
		On("ListObjects", mock.Anything, mock.Anything).
		Return([]string(nil), errors.New("some error")).
		Once()

	_, err := g.GetLogs(ctx, time.Now(), time.Now(), id.NewWorkflowID(), id.NewJobID())
	assert.Error(t, err, "should return an error if listing objects fails")

	mClient.AssertExpectations(t)
	mBucket.AssertExpectations(t)
}

func TestGCSLog_GetLogs_ReadError(t *testing.T) {
	ctx := context.Background()

	mClient := new(mockGCSClient)
	mBucket := new(mockGCSBucket)

	g, _ := NewGCSLog(mClient, "test-bucket")

	mClient.
		On("Bucket", "test-bucket").
		Return(mBucket).
		Once()

	wfID := id.NewWorkflowID()
	jbID := id.NewJobID()

	// We'll look up logs for a single date
	prefix := fmt.Sprintf("artifacts/logs/2025/01/01/%s/%s/", wfID.String(), jbID.String())
	mBucket.
		On("ListObjects", mock.Anything, prefix).
		Return([]string{prefix + "obj.json"}, nil).
		Once()

	// Force ReadObject to fail
	mBucket.
		On("ReadObject", mock.Anything, prefix+"obj.json").
		Return([]byte(nil), errors.New("read error")).
		Once()

	_, err := g.GetLogs(
		ctx,
		time.Date(2025, 1, 1, 0, 0, 0, 0, time.UTC),
		time.Date(2025, 1, 1, 23, 59, 0, 0, time.UTC),
		wfID,
		jbID,
	)

	// Current implementation: skip the failed object, but not fail the entire call
	assert.NoError(t, err, "implementation is skipping errors for individual objects")

	mClient.AssertExpectations(t)
	mBucket.AssertExpectations(t)
}

// Test logs that are before or after the [since, until] window are excluded
func TestGCSLog_GetLogs_TimeFiltering(t *testing.T) {
	ctx := context.Background()

	mClient := new(mockGCSClient)
	mBucket := new(mockGCSBucket)

	g, _ := NewGCSLog(mClient, "test-bucket")

	mClient.
		On("Bucket", "test-bucket").
		Return(mBucket).
		Once()

	since := time.Date(2025, 1, 1, 10, 0, 0, 0, time.UTC)
	until := time.Date(2025, 1, 1, 11, 0, 0, 0, time.UTC)

	wfID := id.NewWorkflowID()
	jbID := id.NewJobID()

	// Daily prefix for 2025-01-01
	prefix := fmt.Sprintf("artifacts/logs/2025/01/01/%s/%s/", wfID.String(), jbID.String())

	// We will have 3 log files:
	// 1) Outside (before since)
	// 2) Within [since, until]
	// 3) Outside (after until)
	entryBefore := LogEntry{
		WorkflowID: wfID.String(),
		JobID:      jbID.String(),
		Timestamp:  time.Date(2025, 1, 1, 9, 59, 59, 0, time.UTC), // before
		LogLevel:   log.LevelInfo,
		Message:    "before",
	}
	entryWithin := LogEntry{
		WorkflowID: wfID.String(),
		JobID:      jbID.String(),
		Timestamp:  time.Date(2025, 1, 1, 10, 30, 0, 0, time.UTC), // within range
		LogLevel:   log.LevelInfo,
		Message:    "within",
	}
	entryAfter := LogEntry{
		WorkflowID: wfID.String(),
		JobID:      jbID.String(),
		Timestamp:  time.Date(2025, 1, 1, 11, 0, 1, 0, time.UTC), // after
		LogLevel:   log.LevelInfo,
		Message:    "after",
	}

	bEntryBefore, _ := json.Marshal(entryBefore)
	bEntryWithin, _ := json.Marshal(entryWithin)
	bEntryAfter, _ := json.Marshal(entryAfter)

	objBefore := prefix + "before.json"
	objWithin := prefix + "within.json"
	objAfter := prefix + "after.json"

	mBucket.
		On("ListObjects", mock.Anything, prefix).
		Return([]string{objBefore, objWithin, objAfter}, nil).
		Once()

	// Return each object in turn
	mBucket.
		On("ReadObject", mock.Anything, objBefore).
		Return(bEntryBefore, nil).
		Once()
	mBucket.
		On("ReadObject", mock.Anything, objWithin).
		Return(bEntryWithin, nil).
		Once()
	mBucket.
		On("ReadObject", mock.Anything, objAfter).
		Return(bEntryAfter, nil).
		Once()

	results, err := g.GetLogs(ctx, since, until, wfID, jbID)
	assert.NoError(t, err)
	assert.Len(t, results, 1, "only the 'within' log should be returned")
	assert.Equal(t, "within", results[0].Message())
}

// Test date range limit (> 30 days) is disallowed
func TestGCSLog_GetLogs_TooLargeDateRange(t *testing.T) {
	ctx := context.Background()

	mClient := new(mockGCSClient)
	mBucket := new(mockGCSBucket)

	g, _ := NewGCSLog(mClient, "test-bucket")

	mClient.
		On("Bucket", "test-bucket").
		Return(mBucket).
		Maybe() // We do not expect calls if the date range is rejected early

	// 40-day range
	since := time.Date(2025, 1, 1, 0, 0, 0, 0, time.UTC)
	until := since.AddDate(0, 0, 40) // 2025-02-10 (40 days after 2025-01-01)

	_, err := g.GetLogs(ctx, since, until, id.NewWorkflowID(), id.NewJobID())
	assert.Error(t, err, "should fail if range spans more than 30 days")
	assert.Contains(t, err.Error(), "date range too large")
}
