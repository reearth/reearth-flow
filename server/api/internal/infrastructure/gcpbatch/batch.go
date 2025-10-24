package gcpbatch

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"regexp"
	"strconv"
	"strings"

	batch "cloud.google.com/go/batch/apiv1"
	batchpb "cloud.google.com/go/batch/apiv1/batchpb"
	"github.com/googleapis/gax-go/v2"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/log"
	"google.golang.org/api/iterator"
)

type BatchConfig struct {
	AllowedLocations                []string
	BinaryPath                      string
	BootDiskSizeGB                  int
	BootDiskType                    string
	ChannelBufferSize               string
	ComputeCpuMilli                 int
	ComputeMemoryMib                int
	FeatureFlushThreshold           string
	ImageURI                        string
	MachineType                     string
	NodeStatusPropagationDelayMS    string
	PubSubEdgePassThroughEventTopic string
	PubSubLogStreamTopic            string
	PubSubJobCompleteTopic          string
	PubSubNodeStatusTopic           string
	PubSubUserFacingLogTopic        string
	ProjectID                       string
	Region                          string
	RustLog                         string
	SAEmail                         string
	TaskCount                       int
	ThreadPoolSize                  string
	CompressIntermediateData        bool
	PersistIngressData              bool
}

type BatchClient interface {
	CreateJob(
		ctx context.Context,
		req *batchpb.CreateJobRequest,
		opts ...gax.CallOption,
	) (*batchpb.Job, error)
	GetJob(
		ctx context.Context,
		req *batchpb.GetJobRequest,
		opts ...gax.CallOption,
	) (*batchpb.Job, error)
	ListJobs(
		ctx context.Context,
		req *batchpb.ListJobsRequest,
		opts ...gax.CallOption,
	) *batch.JobIterator
	DeleteJob(
		ctx context.Context,
		req *batchpb.DeleteJobRequest,
		opts ...gax.CallOption,
	) (*batch.DeleteJobOperation, error)
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

func (b *BatchRepo) SubmitJob(
	ctx context.Context,
	jobID id.JobID,
	workflowsURL, metadataURL string,
	variables map[string]interface{},
	projectID id.ProjectID,
	workspaceID id.WorkspaceID,
) (string, error) {
	formattedJobID := formatJobID(jobID.String())

	jobName := fmt.Sprintf(
		"projects/%s/locations/%s/jobs/%s",
		b.config.ProjectID,
		b.config.Region,
		formattedJobID,
	)
	parent := fmt.Sprintf("projects/%s/locations/%s", b.config.ProjectID, b.config.Region)

	binaryPath := b.config.BinaryPath
	if binaryPath == "" {
		binaryPath = "reearth-flow-worker"
	}

	var varArgs []string
	if len(variables) > 0 {
		for k, v := range variables {
			varArgs = append(varArgs, fmt.Sprintf("--var=%s=%v", k, v))
		}
	}

	varString := strings.Join(varArgs, " ")
	workflowCommand := fmt.Sprintf(
		"%s --workflow %q --metadata-path %q %s",
		binaryPath,
		workflowsURL,
		metadataURL,
		varString,
	)

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

	computeResource := &batchpb.ComputeResource{
		BootDiskMib: int64(b.config.BootDiskSizeGB * 1024),
		CpuMilli:    int64(b.config.ComputeCpuMilli),
		MemoryMib:   int64(b.config.ComputeMemoryMib),
	}

	taskSpec := &batchpb.TaskSpec{
		ComputeResource: computeResource,
		Runnables: []*batchpb.Runnable{
			runnable,
		},
		Environment: &batchpb.Environment{
			Variables: func() map[string]string {
				rustLog := b.config.RustLog
				if rustLog == "" {
					rustLog = "info"
				}
				vars := map[string]string{
					"FLOW_WORKER_ENABLE_JSON_LOG":               "true",
					"FLOW_WORKER_EDGE_PASS_THROUGH_EVENT_TOPIC": b.config.PubSubEdgePassThroughEventTopic,
					"FLOW_WORKER_LOG_STREAM_TOPIC":              b.config.PubSubLogStreamTopic,
					"FLOW_WORKER_JOB_COMPLETE_TOPIC":            b.config.PubSubJobCompleteTopic,
					"FLOW_WORKER_NODE_STATUS_TOPIC":             b.config.PubSubNodeStatusTopic,
					"FLOW_WORKER_USER_FACING_LOG_TOPIC":         b.config.PubSubUserFacingLogTopic,
					"RUST_LOG":                                  rustLog,
					"RUST_BACKTRACE":                            "1",
				}

				// Only set runtime config if values are provided
				if b.config.NodeStatusPropagationDelayMS != "" {
					vars["FLOW_RUNTIME_NODE_STATUS_PROPAGATION_DELAY_MS"] = b.config.NodeStatusPropagationDelayMS
				}
				if b.config.ChannelBufferSize != "" {
					vars["FLOW_RUNTIME_CHANNEL_BUFFER_SIZE"] = b.config.ChannelBufferSize
				}
				if b.config.ThreadPoolSize != "" {
					vars["FLOW_RUNTIME_THREAD_POOL_SIZE"] = b.config.ThreadPoolSize
				}
				if b.config.FeatureFlushThreshold != "" {
					vars["FLOW_RUNTIME_FEATURE_FLUSH_THRESHOLD"] = b.config.FeatureFlushThreshold
				}
				if b.config.CompressIntermediateData {
					vars["FLOW_RUNTIME_COMPRESS_INTERMEDIATE_DATA"] = strconv.FormatBool(b.config.CompressIntermediateData)
				}
				if b.config.PersistIngressData {
					vars["FLOW_RUNTIME_PERSIST_INGRESS_DATA"] = strconv.FormatBool(b.config.PersistIngressData)
				}

				return vars
			}(),
		},
	}

	taskGroup := &batchpb.TaskGroup{
		TaskCount: int64(b.config.TaskCount),
		TaskSpec:  taskSpec,
	}

	bootDisk := &batchpb.AllocationPolicy_Disk{
		Type:   b.config.BootDiskType,
		SizeGb: int64(b.config.BootDiskSizeGB),
	}

	instancePolicy := &batchpb.AllocationPolicy_InstancePolicy{
		ProvisioningModel: batchpb.AllocationPolicy_STANDARD,
		MachineType:       b.config.MachineType,
		BootDisk:          bootDisk,
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
		ServiceAccount: &batchpb.ServiceAccount{
			Email: b.config.SAEmail,
		},
	}

	if len(b.config.AllowedLocations) > 0 {
		allocationPolicy.Location = &batchpb.AllocationPolicy_LocationPolicy{
			AllowedLocations: b.config.AllowedLocations,
		}
	}

	labels := map[string]string{
		"project_id":  projectID.String(),
		"original_id": jobID.String(),
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
		JobId:  formattedJobID,
		Job:    job,
	}

	resp, err := b.client.CreateJob(ctx, req)
	if err != nil {
		log.Debugfc(ctx, "[Batch] Error creating job: %v", err)
		return "", fmt.Errorf("failed to create job: %v", err)
	}

	log.Debugfc(ctx, "[Batch] Job created successfully: name=%s, uid=%s", resp.Name, resp.Uid)

	if resp.Name == "" {
		log.Warnfc(ctx, "[Batch] Empty job name returned from GCP API")

		log.Debugfc(ctx, "[Batch] Using fallback name: %s", jobName)
		return jobName, nil
	}

	return resp.Name, nil
}

func (b *BatchRepo) GetJobStatus(ctx context.Context, jobName string) (gateway.JobStatus, error) {
	log.Debugfc(ctx, "GetJobStatus: name=%s, config.ProjectID=%s, config.Region=%s",
		jobName, b.config.ProjectID, b.config.Region)

	req := &batchpb.GetJobRequest{
		Name: jobName,
	}

	job, err := b.client.GetJob(ctx, req)
	if err != nil {
		if strings.Contains(err.Error(), "NotFound") || strings.Contains(err.Error(), "404") {
			log.Debugfc(ctx, "Job not found (possibly deleted): %s", jobName)
			return gateway.JobStatusCancelled, nil
		}

		if strings.Contains(err.Error(), "RESOURCE_PROJECT_INVALID") {
			log.Debugfc(ctx, "Detected project invalid error, inspecting job name: %s", jobName)

			parts := strings.Split(jobName, "/")
			jobID := parts[len(parts)-1]

			fixedName := fmt.Sprintf("projects/%s/locations/%s/jobs/%s",
				b.config.ProjectID, b.config.Region, jobID)

			log.Debugfc(ctx, "Retrying with name: %s", fixedName)

			retryReq := &batchpb.GetJobRequest{
				Name: fixedName,
			}

			job, err = b.client.GetJob(ctx, retryReq)
			if err != nil {
				if strings.Contains(err.Error(), "NotFound") ||
					strings.Contains(err.Error(), "404") {
					log.Debugfc(ctx, "Job not found after retry (possibly deleted): %s", fixedName)
					return gateway.JobStatusCancelled, nil
				}
				log.Debugfc(ctx, "Retry failed: %v", err)
				return gateway.JobStatusUnknown, fmt.Errorf("failed to get job status: %v", err)
			}

			status := convertGCPStatusToGatewayStatus(job.Status.State)
			return status, nil
		}

		return gateway.JobStatusUnknown, fmt.Errorf("failed to get job status: %v", err)
	}

	status := convertGCPStatusToGatewayStatus(job.Status.State)
	return status, nil
}

func (b *BatchRepo) Close() error {
	return b.client.Close()
}

func (b *BatchRepo) ListJobs(
	ctx context.Context,
	projectID id.ProjectID,
) ([]gateway.JobInfo, error) {
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
	case batchpb.JobStatus_SCHEDULED:
		return gateway.JobStatusPending
	case batchpb.JobStatus_RUNNING:
		return gateway.JobStatusRunning
	case batchpb.JobStatus_SUCCEEDED:
		return gateway.JobStatusCompleted
	case batchpb.JobStatus_FAILED:
		return gateway.JobStatusFailed
	case batchpb.JobStatus_DELETION_IN_PROGRESS:
		return gateway.JobStatusCancelled
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
