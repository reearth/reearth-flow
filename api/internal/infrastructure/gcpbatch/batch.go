package gcpbatch

import (
	"context"
	"fmt"

	batch "cloud.google.com/go/batch/apiv1"
	batchpb "cloud.google.com/go/batch/apiv1/batchpb"
	"github.com/googleapis/gax-go/v2"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/log"
	"google.golang.org/api/iterator"
)

type BatchConfig struct {
	ProjectID string
	Region    string
	ImageURI  string
}

type BatchClient interface {
	CreateJob(ctx context.Context, req *batchpb.CreateJobRequest, opts ...gax.CallOption) (*batchpb.Job, error)
	GetJob(ctx context.Context, req *batchpb.GetJobRequest, opts ...gax.CallOption) (*batchpb.Job, error)
	ListJobs(ctx context.Context, req *batchpb.ListJobsRequest, opts ...gax.CallOption) *batch.JobIterator
	DeleteJob(ctx context.Context, req *batchpb.DeleteJobRequest, opts ...gax.CallOption) (*batch.DeleteJobOperation, error)
	Close() error
}

type BatchRepo struct {
	client BatchClient
	config BatchConfig
}

func NewBatch(ctx context.Context, config BatchConfig) (gateway.Batch, error) {
	client, err := batch.NewClient(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to create batch client: %v", err)
	}

	return &BatchRepo{
		client: client,
		config: config,
	}, nil
}

func (b *BatchRepo) SubmitJob(ctx context.Context, jobID id.JobID, workflowsURL, metadataURL string, projectID id.ProjectID) (string, error) {
	jobName := fmt.Sprintf("projects/%s/locations/%s/jobs/%s", b.config.ProjectID, b.config.Region, jobID)
	parent := fmt.Sprintf("projects/%s/locations/%s", b.config.ProjectID, b.config.Region)
	workflowCommand := fmt.Sprintf("echo %q | /bin/reearth-flow run --workflow - --metadata-path %q", workflowsURL, metadataURL)
	commands := []string{
		"/bin/sh",
		"-c",
		workflowCommand,
	}

	runnableContainer := &batchpb.Runnable_Container{
		ImageUri: b.config.ImageURI,
		Commands: commands,
	}

	runnable := &batchpb.Runnable{
		Executable: &batchpb.Runnable_Container_{
			Container: runnableContainer,
		},
		DisplayName:      "Run reearth-flow workflow with metadata",
		IgnoreExitStatus: false,
		Background:       false,
		AlwaysRun:        false,
	}

	taskSpec := &batchpb.TaskSpec{
		Runnables: []*batchpb.Runnable{
			runnable,
		},
	}

	taskGroup := &batchpb.TaskGroup{
		TaskCount: 1,
		TaskSpec:  taskSpec,
	}

	instancePolicy := &batchpb.AllocationPolicy_InstancePolicy{
		ProvisioningModel: batchpb.AllocationPolicy_STANDARD,
		MachineType:       "e2-standard-4",
	}

	instancePolicyOrTemplate := &batchpb.AllocationPolicy_InstancePolicyOrTemplate{
		PolicyTemplate: &batchpb.AllocationPolicy_InstancePolicyOrTemplate_Policy{
			Policy: instancePolicy,
		},
		InstallGpuDrivers: false,
	}

	allocationPolicy := &batchpb.AllocationPolicy{
		Instances: []*batchpb.AllocationPolicy_InstancePolicyOrTemplate{
			instancePolicyOrTemplate,
		},
	}

	labels := map[string]string{
		"workflow_url": workflowsURL,
		"metadata_url": metadataURL,
		"project_id":   projectID.String(),
	}

	logsPolicy := &batchpb.LogsPolicy{
		Destination: batchpb.LogsPolicy_CLOUD_LOGGING,
	}

	job := &batchpb.Job{
		Name:             jobName,
		TaskGroups:       []*batchpb.TaskGroup{taskGroup},
		AllocationPolicy: allocationPolicy,
		Labels:           labels,
		LogsPolicy:       logsPolicy,
	}

	req := &batchpb.CreateJobRequest{
		Parent: parent,
		JobId:  jobID.String(),
		Job:    job,
	}

	resp, err := b.client.CreateJob(ctx, req)
	if err != nil {
		log.Errorfc(ctx, "gcpbatch: failed to create job: %v", err)
		return "", fmt.Errorf("failed to create job: %v", err)
	}

	return resp.Name, nil
}

func (b *BatchRepo) GetJobStatus(ctx context.Context, jobName string) (gateway.JobStatus, error) {
	req := &batchpb.GetJobRequest{
		Name: jobName,
	}

	job, err := b.client.GetJob(ctx, req)
	if err != nil {
		log.Errorfc(ctx, "gcpbatch: failed to get job status: %v", err)
		return gateway.JobStatusUnknown, fmt.Errorf("failed to get job status: %v", err)
	}

	return convertGCPStatusToGatewayStatus(job.Status.State), nil
}

func (b *BatchRepo) Close() error {
	return b.client.Close()
}

func (b *BatchRepo) ListJobs(ctx context.Context, projectID id.ProjectID) ([]gateway.JobInfo, error) {
	req := &batchpb.ListJobsRequest{
		Parent: fmt.Sprintf("projects/%s/locations/%s", b.config.ProjectID, b.config.Region),
		Filter: fmt.Sprintf("labels.project_id=%s", projectID.String()),
	}

	it := b.client.ListJobs(ctx, req)
	var jobs []gateway.JobInfo
	for {
		job, err := it.Next()
		if err == iterator.Done {
			break
		}
		if err != nil {
			log.Errorfc(ctx, "gcpbatch: failed to list jobs: %v", err)
			return nil, fmt.Errorf("failed to list jobs: %v", err)
		}
		jobID, err := id.JobIDFrom(job.Uid)
		if err != nil {
			return nil, fmt.Errorf("failed to parse job ID: %v", err)
		}
		jobs = append(jobs, gateway.JobInfo{
			ID:     jobID,
			Name:   job.Name,
			Status: convertGCPStatusToGatewayStatus(job.Status.State),
		})
	}

	return jobs, nil
}

func (b *BatchRepo) CancelJob(ctx context.Context, jobName string) error {
	req := &batchpb.DeleteJobRequest{
		Name: jobName,
	}

	_, err := b.client.DeleteJob(ctx, req)
	if err != nil {
		log.Errorfc(ctx, "gcpbatch: failed to cancel job: %v", err)
		return fmt.Errorf("failed to cancel job: %v", err)
	}

	return nil
}

func convertGCPStatusToGatewayStatus(gcpStatus batchpb.JobStatus_State) gateway.JobStatus {
	switch gcpStatus {
	case batchpb.JobStatus_STATE_UNSPECIFIED:
		return gateway.JobStatusUnknown
	case batchpb.JobStatus_QUEUED:
		return gateway.JobStatusPending
	case batchpb.JobStatus_RUNNING:
		return gateway.JobStatusRunning
	case batchpb.JobStatus_SUCCEEDED:
		return gateway.JobStatusCompleted
	case batchpb.JobStatus_FAILED:
		return gateway.JobStatusFailed
	default:
		return gateway.JobStatusUnknown
	}
}
