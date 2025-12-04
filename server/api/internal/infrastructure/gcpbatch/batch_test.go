package gcpbatch

import (
	"context"
	"testing"

	batch "cloud.google.com/go/batch/apiv1"
	batchpb "cloud.google.com/go/batch/apiv1/batchpb"
	"github.com/googleapis/gax-go/v2"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

type mockBatchClient struct {
	mock.Mock
}

func (m *mockBatchClient) CreateJob(ctx context.Context, req *batchpb.CreateJobRequest, opts ...gax.CallOption) (*batchpb.Job, error) {
	args := m.Called(ctx, req)
	return args.Get(0).(*batchpb.Job), args.Error(1)
}

func (m *mockBatchClient) GetJob(ctx context.Context, req *batchpb.GetJobRequest, opts ...gax.CallOption) (*batchpb.Job, error) {
	args := m.Called(ctx, req)
	return args.Get(0).(*batchpb.Job), args.Error(1)
}

func (m *mockBatchClient) ListJobs(ctx context.Context, req *batchpb.ListJobsRequest, opts ...gax.CallOption) *batch.JobIterator {
	args := m.Called(ctx, req)
	return args.Get(0).(*batch.JobIterator)
}

func (m *mockBatchClient) DeleteJob(ctx context.Context, req *batchpb.DeleteJobRequest, opts ...gax.CallOption) (*batch.DeleteJobOperation, error) {
	args := m.Called(ctx, req)
	return args.Get(0).(*batch.DeleteJobOperation), args.Error(1)
}

func (m *mockBatchClient) Close() error {
	args := m.Called()
	return args.Error(0)
}

// type mockJobIterator struct {
// 	jobs    []*batchpb.Job
// 	current int
// }

// func (m *mockJobIterator) Next() (*batchpb.Job, error) {
// 	if m.current >= len(m.jobs) {
// 		return nil, iterator.Done
// 	}
// 	job := m.jobs[m.current]
// 	m.current++
// 	return job, nil
// }

func TestBatchRepo_SubmitJob(t *testing.T) {
	ctx := context.Background()
	mockClient := new(mockBatchClient)
	batchRepo := &BatchRepo{
		client: mockClient,
		config: BatchConfig{
			ProjectID: "test-project",
			Region:    "us-central1",
			ImageURI:  "gcr.io/test-project/reearth-flow:latest",
		},
	}

	jobID, _ := id.JobIDFrom("test-job-id")
	projectID, _ := id.ProjectIDFrom("test-project-id")
	workspaceID, _ := accountsid.WorkspaceIDFrom("test-workspace-id")
	workflowURL := "gs://test-bucket/test-workflow.yaml"
	metadataURL := "gs://test-bucket/test-metadata.json"
	var variables map[string]string

	expectedJobName := "projects/test-project/locations/us-central1/jobs/test-job-id"

	mockClient.On("CreateJob", ctx, mock.AnythingOfType("*batchpb.CreateJobRequest")).Return(&batchpb.Job{Name: expectedJobName}, nil)

	jobName, err := batchRepo.SubmitJob(ctx, jobID, workflowURL, metadataURL, variables, projectID, accountsid.WorkspaceID(workspaceID))

	assert.NoError(t, err)
	assert.Equal(t, expectedJobName, jobName)
	mockClient.AssertExpectations(t)
}

func TestBatchRepo_GetJobStatus(t *testing.T) {
	ctx := context.Background()
	mockClient := new(mockBatchClient)
	batchRepo := &BatchRepo{
		client: mockClient,
		config: BatchConfig{},
	}

	jobName := "projects/test-project/locations/us-central1/jobs/test-job-id"

	mockClient.On("GetJob", ctx, &batchpb.GetJobRequest{Name: jobName}).Return(&batchpb.Job{
		Status: &batchpb.JobStatus{State: batchpb.JobStatus_RUNNING},
	}, nil)

	status, err := batchRepo.GetJobStatus(ctx, jobName)

	assert.NoError(t, err)
	assert.Equal(t, gateway.JobStatusRunning, status)
	mockClient.AssertExpectations(t)
}

func TestBatchRepo_Close(t *testing.T) {
	mockClient := new(mockBatchClient)
	batchRepo := &BatchRepo{
		client: mockClient,
		config: BatchConfig{},
	}

	mockClient.On("Close").Return(nil)

	err := batchRepo.Close()

	assert.NoError(t, err)
	mockClient.AssertExpectations(t)
}

func TestConvertGCPStatusToGatewayStatus(t *testing.T) {
	testCases := []struct {
		name         string
		gcpStatus    batchpb.JobStatus_State
		expectStatus gateway.JobStatus
	}{
		{"Unspecified", batchpb.JobStatus_STATE_UNSPECIFIED, gateway.JobStatusUnknown},
		{"Queued", batchpb.JobStatus_QUEUED, gateway.JobStatusPending},
		{"Scheduled", batchpb.JobStatus_SCHEDULED, gateway.JobStatusPending},
		{"Running", batchpb.JobStatus_RUNNING, gateway.JobStatusRunning},
		{"Succeeded", batchpb.JobStatus_SUCCEEDED, gateway.JobStatusCompleted},
		{"Failed", batchpb.JobStatus_FAILED, gateway.JobStatusFailed},
		{"Unknown", batchpb.JobStatus_State(999), gateway.JobStatusUnknown},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			result := convertGCPStatusToGatewayStatus(tc.gcpStatus)
			assert.Equal(t, tc.expectStatus, result)
		})
	}
}
