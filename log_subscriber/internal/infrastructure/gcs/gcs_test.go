package gcs

import (
	"bytes"
	"context"
	"testing"
	"time"

	domainLog "github.com/reearth/reearth-flow/log-subscriber/pkg/log"
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

func (m *mockGCSBucket) Object(objName string) GCSObject {
	args := m.Called(objName)
	return args.Get(0).(GCSObject)
}

type mockGCSObject struct {
	mock.Mock
}

func (m *mockGCSObject) NewWriter(ctx context.Context) GCSWriter {
	args := m.Called(ctx)
	return args.Get(0).(GCSWriter)
}

type mockGCSWriter struct {
	mock.Mock
	buf bytes.Buffer
}

func (m *mockGCSWriter) Write(p []byte) (int, error) {
	args := m.Called(p)
	n := args.Int(0)
	err := args.Error(1)
	m.buf.Write(p)
	return n, err
}
func (m *mockGCSWriter) Close() error {
	args := m.Called()
	return args.Error(0)
}
func (m *mockGCSWriter) SetContentType(ct string) {
	m.Called(ct)
}

func TestGCSStorage_SaveLogToGCS(t *testing.T) {
	ctx := context.Background()

	mClient := new(mockGCSClient)
	mBucket := new(mockGCSBucket)
	mObject := new(mockGCSObject)
	mWriter := new(mockGCSWriter)

	const bucketName = "reearth-flow-oss-bucket"
	gcsStorage := NewGCSStorage(mClient, bucketName)

	event := &domainLog.LogEvent{
		WorkflowID: "workflow-123",
		JobID:      "job-abc",
		Timestamp:  time.Date(2025, 1, 11, 9, 12, 54, 0, time.UTC),
		LogLevel:   domainLog.LogLevelInfo,
		Message:    "Hello from test",
	}

	expectedFilePath := "artifacts/logs/2025/01/11/workflow-123/job-abc/2025-01-11T09:12:54.000000Z.json"

	mClient.
		On("Bucket", bucketName).
		Return(mBucket)

	mBucket.
		On("Object", expectedFilePath).
		Return(mObject)

	mObject.
		On("NewWriter", mock.Anything).
		Return(mWriter)

	mWriter.
		On("SetContentType", "application/json").
		Return()

	mWriter.
		On("Write", mock.Anything).
		Return(100, nil)

	mWriter.
		On("Close").
		Return(nil)

	err := gcsStorage.SaveLogToGCS(ctx, event)
	assert.NoError(t, err)

	mClient.AssertExpectations(t)
	mBucket.AssertExpectations(t)
	mObject.AssertExpectations(t)
	mWriter.AssertExpectations(t)
}
