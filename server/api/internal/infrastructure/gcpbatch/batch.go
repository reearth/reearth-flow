package gcpbatch

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"regexp"
	"strings"

	batch "cloud.google.com/go/batch/apiv1"
	batchpb "cloud.google.com/go/batch/apiv1/batchpb"
	"github.com/googleapis/gax-go/v2"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/log"
	"google.golang.org/api/iterator"
)

type BatchConfig struct {
	BinaryPath     string
	BootDiskSizeGB int
	BootDiskType   string
	ImageURI       string
	MachineType    string
	ProjectID      string
	Region         string
	SAEmail        string
	TaskCount      int
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

func (b *BatchRepo) SubmitJob(ctx context.Context, jobID id.JobID, workflowsURL, metadataURL string, variables map[string]interface{}, projectID id.ProjectID, workspaceID accountdomain.WorkspaceID) (string, error) {
	log.Debugfc(ctx, "gcpbatch: starting job submission with jobID=%s projectID=%s workspaceID=%s", jobID, projectID, workspaceID)

	formattedJobID := formatJobID(jobID.String())
	log.Debugfc(ctx, "gcpbatch: formatted jobID from %s to %s", jobID, formattedJobID)

	jobName := fmt.Sprintf("projects/%s/locations/%s/jobs/%s", b.config.ProjectID, b.config.Region, formattedJobID)
	parent := fmt.Sprintf("projects/%s/locations/%s", b.config.ProjectID, b.config.Region)
	log.Debugfc(ctx, "gcpbatch: constructed job name=%s parent=%s", jobName, parent)

	binaryPath := b.config.BinaryPath
	if binaryPath == "" {
		binaryPath = "reearth-flow-worker"
		log.Debugfc(ctx, "gcpbatch: using default binary path=%s", binaryPath)
	} else {
		log.Debugfc(ctx, "gcpbatch: using configured binary path=%s", binaryPath)
	}

	var varArgs []string
	if len(variables) > 0 {
		log.Debugfc(ctx, "gcpbatch: processing %d variables", len(variables))
		for k, v := range variables {
			varArgs = append(varArgs, fmt.Sprintf("--var=%s=%v", k, v))
			log.Debugfc(ctx, "gcpbatch: added variable %s=%v", k, v)
		}
	} else {
		log.Debugfc(ctx, "gcpbatch: no variables provided")
	}

	varString := strings.Join(varArgs, " ")
	workflowCommand := fmt.Sprintf(
		"%s --workflow %q --metadata-path %q --pubsub-backend noop %s",
		binaryPath,
		workflowsURL,
		metadataURL,
		varString,
	)
	log.Debugfc(ctx, "gcpbatch: constructed workflow command: %s", workflowCommand)

	commands := []string{
		"/bin/sh",
		"-c",
		workflowCommand,
	}

	runnableContainer := &batchpb.Runnable_Container{
		ImageUri: b.config.ImageURI,
		Commands: commands,
	}
	log.Debugfc(ctx, "gcpbatch: created container config with image=%s and JSON logging enabled", b.config.ImageURI)

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
		Environment: &batchpb.Environment{
			Variables: map[string]string{
				"FLOW_RUNTIME_FEATURE_WRITER_DISABLE": "true",
				"FLOW_WORKER_ENABLE_JSON_LOG":         "true",
			},
		},
	}

	taskGroup := &batchpb.TaskGroup{
		TaskCount: int64(b.config.TaskCount),
		TaskSpec:  taskSpec,
	}
	log.Debugfc(ctx, "gcpbatch: configured task group with count=%d", taskGroup.TaskCount)

	bootDisk := &batchpb.AllocationPolicy_Disk{
		Type:   b.config.BootDiskType,
		SizeGb: int64(b.config.BootDiskSizeGB),
	}

	instancePolicy := &batchpb.AllocationPolicy_InstancePolicy{
		ProvisioningModel: batchpb.AllocationPolicy_STANDARD,
		MachineType:       b.config.MachineType,
		BootDisk:          bootDisk,
	}
	log.Debugfc(ctx, "gcpbatch: configured instance policy with machine=%s", instancePolicy.MachineType)

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
		ServiceAccount: &batchpb.ServiceAccount{
			Email: b.config.SAEmail,
		},
	}
	log.Debugfc(ctx, "gcpbatch: configured allocation policy with service account=%s", b.config.SAEmail)

	labels := map[string]string{
		"project_id":  projectID.String(),
		"original_id": jobID.String(),
	}
	log.Debugfc(ctx, "gcpbatch: set job labels: %v", labels)

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
		JobId:  formattedJobID,
		Job:    job,
	}
	log.Debugfc(ctx, "gcpbatch: submitting job create request for jobID=%s", formattedJobID)

	resp, err := b.client.CreateJob(ctx, req)
	if err != nil {
		log.Errorfc(ctx, "gcpbatch: failed to create job: %v", err)
		return "", fmt.Errorf("failed to create job: %v", err)
	}
	log.Debugfc(ctx, "gcpbatch: successfully created job with name=%s", resp.Name)

	return resp.Name, nil
}

func (b *BatchRepo) GetJobStatus(ctx context.Context, jobName string) (gateway.JobStatus, error) {
	log.Debugfc(ctx, "gcpbatch: getting status for job=%s", jobName)

	req := &batchpb.GetJobRequest{
		Name: jobName,
	}

	job, err := b.client.GetJob(ctx, req)
	if err != nil {
		log.Errorfc(ctx, "gcpbatch: failed to get job status: %v", err)
		return gateway.JobStatusUnknown, fmt.Errorf("failed to get job status: %v", err)
	}

	status := convertGCPStatusToGatewayStatus(job.Status.State)
	log.Debugfc(ctx, "gcpbatch: job=%s state=%s converted_status=%s", jobName, job.Status.State, status)

	return status, nil
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

func formatJobID(jobID string) string {
	if regexp.MustCompile(`^[0-9]`).MatchString(jobID) {
		jobID = "j-" + jobID
	}

	jobID = strings.ToLower(jobID)
	jobID = regexp.MustCompile(`[^a-z0-9-]`).ReplaceAllString(jobID, "-")

	jobID = strings.TrimSuffix(jobID, "-")

	if len(jobID) > 63 {
		hash := sha256.Sum256([]byte(jobID))
		hashStr := hex.EncodeToString(hash[:])[:8]
		jobID = jobID[:54] + "-" + hashStr
	}

	return jobID
}
