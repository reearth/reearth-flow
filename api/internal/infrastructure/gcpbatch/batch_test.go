package gcpbatch

import (
	"context"
	"testing"

	batch "cloud.google.com/go/batch/apiv1"
	batchpb "cloud.google.com/go/batch/apiv1/batchpb"
	"github.com/googleapis/gax-go/v2"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"google.golang.org/api/iterator"
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

type mockJobIterator struct {
    jobs []*batchpb.Job
    current int
}

func (m *mockJobIterator) Next() (*batchpb.Job, error) {
    if m.current >= len(m.jobs) {
        return nil, iterator.Done
    }
    job := m.jobs[m.current]
    m.current++
    return job, nil
}

func TestNewBatch(t *testing.T) {
	ctx := context.Background()
	config := Config{
		ProjectID: "test-project",
		Region:    "us-central1",
		ImageURI:  "gcr.io/test-project/reearth-flow:latest",
	}

	batch, err := NewBatch(ctx, config)

	assert.NoError(t, err)
	assert.NotNil(t, batch)
}

func TestBatchRepo_SubmitJob(t *testing.T) {
	ctx := context.Background()
	mockClient := new(mockBatchClient)
	batchRepo := &BatchRepo{
		client: mockClient,
		config: Config{
			ProjectID: "test-project",
			Region:    "us-central1",
			ImageURI:  "gcr.io/test-project/reearth-flow:latest",
		},
	}

	jobID, _ := id.JobIDFrom("test-job-id")
	projectID, _ := id.ProjectIDFrom("test-project-id")
	workflowID, _ := id.WorkflowIDFrom("test-workflow-id")
	yamlString := "test-yaml-content"
	testWorkflow := &workflow.Workflow{
		ID:         workflowID,
		YamlString: &yamlString,
	}

	expectedJobName := "projects/test-project/locations/us-central1/jobs/test-job-id"

	mockClient.On("CreateJob", ctx, mock.AnythingOfType("*batchpb.CreateJobRequest")).Return(&batchpb.Job{Name: expectedJobName}, nil)

	jobName, err := batchRepo.SubmitJob(ctx, jobID, testWorkflow, projectID)

	assert.NoError(t, err)
	assert.Equal(t, expectedJobName, jobName)
	mockClient.AssertExpectations(t)
}

func TestBatchRepo_GetJobStatus(t *testing.T) {
	ctx := context.Background()
	mockClient := new(mockBatchClient)
	batchRepo := &BatchRepo{
		client: mockClient,
		config: Config{},
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

func TestBatchRepo_ListJobs(t *testing.T) {
    ctx := context.Background()
    mockClient := new(mockBatchClient)
    batchRepo := &BatchRepo{
        client: mockClient,
        config: Config{
            ProjectID: "test-project",
            Region:    "us-central1",
        },
    }

    projectID, _ := id.ProjectIDFrom("test-project-id")

    job1 := &batchpb.Job{Name: "job1", Uid: "job-id-1", Status: &batchpb.JobStatus{State: batchpb.JobStatus_RUNNING}}
    job2 := &batchpb.Job{Name: "job2", Uid: "job-id-2", Status: &batchpb.JobStatus{State: batchpb.JobStatus_SUCCEEDED}}

    mockIterator := &mockJobIterator{
        jobs: []*batchpb.Job{job1, job2},
    }

    mockClient.On("ListJobs", ctx, mock.AnythingOfType("*batchpb.ListJobsRequest")).Return(mockIterator)

    jobs, err := batchRepo.ListJobs(ctx, projectID)

    assert.NoError(t, err)
    assert.Len(t, jobs, 2)
    assert.Equal(t, "job1", jobs[0].Name)
    assert.Equal(t, gateway.JobStatusRunning, jobs[0].Status)
    assert.Equal(t, "job2", jobs[1].Name)
    assert.Equal(t, gateway.JobStatusCompleted, jobs[1].Status)

    mockClient.AssertExpectations(t)
}

func TestBatchRepo_CancelJob(t *testing.T) {
	ctx := context.Background()
	mockClient := new(mockBatchClient)
	batchRepo := &BatchRepo{
		client: mockClient,
		config: Config{},
	}

	jobName := "projects/test-project/locations/us-central1/jobs/test-job-id"

	mockClient.On("DeleteJob", ctx, &batchpb.DeleteJobRequest{Name: jobName}).Return(&batchpb.Job{}, nil)

	err := batchRepo.CancelJob(ctx, jobName)

	assert.NoError(t, err)
	mockClient.AssertExpectations(t)
}

func TestBatchRepo_Close(t *testing.T) {
	mockClient := new(mockBatchClient)
	batchRepo := &BatchRepo{
		client: mockClient,
		config: Config{},
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
