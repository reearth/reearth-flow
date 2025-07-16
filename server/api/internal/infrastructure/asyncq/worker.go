package asyncq

import (
	"context"
	"fmt"
	"time"

	"github.com/hibiken/asynq"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/log"
)

type AsyncqWorker struct {
	server       *asynq.Server
	mux          *asynq.ServeMux
	config       *Config
	jobRepo      repo.Job
	fileGateway  gateway.File
	batchGateway gateway.Batch
}

func NewAsyncqWorker(
	config *Config,
	jobRepo repo.Job,
	fileGateway gateway.File,
	batchGateway gateway.Batch,
) *AsyncqWorker {
	redisOpt := config.GetRedisClientOpt()

	server := asynq.NewServer(
		redisOpt,
		asynq.Config{
			Concurrency:    config.Concurrency,
			Queues:         config.Queues,
			RetryDelayFunc: config.RetryDelayFunc,
		},
	)

	mux := asynq.NewServeMux()

	worker := &AsyncqWorker{
		server:       server,
		mux:          mux,
		config:       config,
		jobRepo:      jobRepo,
		fileGateway:  fileGateway,
		batchGateway: batchGateway,
	}

	worker.registerHandlers()

	return worker
}

func (w *AsyncqWorker) registerHandlers() {
	w.mux.HandleFunc(TypeWorkflowJob, w.HandleWorkflowJob)
}

func (w *AsyncqWorker) HandleWorkflowJob(ctx context.Context, task *asynq.Task) error {
	payload, err := ParseWorkflowJobPayload(task)
	if err != nil {
		return fmt.Errorf("failed to parse workflow job payload: %w", err)
	}

	log.Infof("Processing workflow job: %s", payload.JobID)

	jobID, err := id.JobIDFrom(payload.JobID)
	if err != nil {
		return fmt.Errorf("invalid job ID: %w", err)
	}

	projectID, err := id.ProjectIDFrom(payload.ProjectID)
	if err != nil {
		return fmt.Errorf("invalid project ID: %w", err)
	}

	workspaceID, err := accountdomain.WorkspaceIDFrom(payload.WorkspaceID)
	if err != nil {
		return fmt.Errorf("invalid workspace ID: %w", err)
	}

	deploymentID, err := id.DeploymentIDFrom(payload.DeploymentID)
	if err != nil {
		return fmt.Errorf("invalid deployment ID: %w", err)
	}

	j, err := w.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		return fmt.Errorf("job not found: %w", err)
	}

	j.SetStatus(job.StatusRunning)
	if err := w.jobRepo.Save(ctx, j); err != nil {
		log.Warnf("Failed to update job status to running: %v", err)
	}

	err = w.executeWorkflow(ctx, payload, jobID, projectID, workspaceID, deploymentID)

	if err != nil {
		j.SetStatus(job.StatusFailed)
		now := time.Now()
		j.SetCompletedAt(&now)
		log.Errorf("Workflow job %s failed: %v", payload.JobID, err)
	} else {
		j.SetStatus(job.StatusCompleted)
		now := time.Now()
		j.SetCompletedAt(&now)
		log.Infof("Workflow job %s completed successfully", payload.JobID)
	}

	// Save final job state
	if saveErr := w.jobRepo.Save(ctx, j); saveErr != nil {
		log.Errorf("Failed to save job final state: %v", saveErr)
	}

	// Update job artifacts
	if err := w.updateJobArtifacts(ctx, j); err != nil {
		log.Warnf("Failed to update job artifacts: %v", err)
	}

	return err
}

func (w *AsyncqWorker) executeWorkflow(
	ctx context.Context,
	payload WorkflowJobPayload,
	jobID id.JobID,
	projectID id.ProjectID,
	workspaceID accountdomain.WorkspaceID,
	deploymentID id.DeploymentID,
) error {
	log.Infof("Starting workflow execution for job %s via GCP Batch", jobID)
	log.Infof("Workflow URL: %s", payload.WorkflowURL)
	log.Infof("Metadata URL: %s", payload.MetadataURL)

	gcpJobName, err := w.batchGateway.SubmitJob(
		ctx,
		jobID,
		payload.WorkflowURL,
		payload.MetadataURL,
		payload.Variables,
		projectID,
		workspaceID,
	)
	if err != nil {
		return fmt.Errorf("failed to submit job to GCP Batch: %w", err)
	}

	log.Infof("Job %s submitted to GCP Batch with name: %s", jobID, gcpJobName)

	j, err := w.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		return fmt.Errorf("failed to find job: %w", err)
	}
	j.SetGCPJobID(gcpJobName)
	if err := w.jobRepo.Save(ctx, j); err != nil {
		log.Warnf("Failed to update job with GCP job name: %v", err)
	}

	return w.monitorGCPBatchJob(ctx, gcpJobName, jobID)
}

func (w *AsyncqWorker) monitorGCPBatchJob(ctx context.Context, gcpJobName string, jobID id.JobID) error {
	ticker := time.NewTicker(10 * time.Second)
	defer ticker.Stop()

	log.Infof("Starting monitoring of GCP Batch job %s", gcpJobName)

	for {
		select {
		case <-ctx.Done():
			log.Infof("Context cancelled for job %s", jobID)
			return ctx.Err()
		case <-ticker.C:
			status, err := w.batchGateway.GetJobStatus(ctx, gcpJobName)
			if err != nil {
				log.Errorf("Failed to get GCP Batch job status: %v", err)
				continue
			}

			log.Debugf("GCP Batch job %s status: %s", gcpJobName, status)

			if err := w.updateJobStatus(ctx, jobID, status); err != nil {
				log.Errorf("Failed to update job status: %v", err)
			}

			switch status {
			case gateway.JobStatusCompleted:
				log.Infof("GCP Batch job %s completed successfully", gcpJobName)
				return nil
			case gateway.JobStatusFailed:
				log.Errorf("GCP Batch job %s failed", gcpJobName)
				return fmt.Errorf("GCP Batch job failed")
			case gateway.JobStatusCancelled:
				log.Infof("GCP Batch job %s was cancelled", gcpJobName)
				return fmt.Errorf("GCP Batch job was cancelled")
			default:
				continue
			}
		}
	}
}

func (w *AsyncqWorker) updateJobStatus(ctx context.Context, jobID id.JobID, status gateway.JobStatus) error {
	j, err := w.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		return err
	}

	var jobStatus job.Status
	switch status {
	case gateway.JobStatusPending:
		jobStatus = job.StatusPending
	case gateway.JobStatusRunning:
		jobStatus = job.StatusRunning
	case gateway.JobStatusCompleted:
		jobStatus = job.StatusCompleted
	case gateway.JobStatusFailed:
		jobStatus = job.StatusFailed
	case gateway.JobStatusCancelled:
		jobStatus = job.StatusCancelled
	default:
		jobStatus = job.StatusPending
	}

	if j.Status() != jobStatus {
		j.SetStatus(jobStatus)
		if jobStatus == job.StatusCompleted || jobStatus == job.StatusFailed || jobStatus == job.StatusCancelled {
			now := time.Now()
			j.SetCompletedAt(&now)
		}
		return w.jobRepo.Save(ctx, j)
	}

	return nil
}

func (w *AsyncqWorker) updateJobArtifacts(ctx context.Context, j *job.Job) error {
	jobID := j.ID().String()

	outputs, err := w.fileGateway.ListJobArtifacts(ctx, jobID)
	if err != nil {
		return fmt.Errorf("failed to list job artifacts: %w", err)
	}
	j.SetOutputURLs(outputs)

	logURL := w.fileGateway.GetJobLogURL(jobID)
	if logURL != "" {
		j.SetLogsURL(logURL)
	}

	workerLogURL := w.fileGateway.GetJobWorkerLogURL(jobID)
	if workerLogURL != "" {
		j.SetWorkerLogsURL(workerLogURL)
	}

	return nil
}

func (w *AsyncqWorker) Start() error {
	log.Info("Starting asyncq worker server")
	return w.server.Start(w.mux)
}

func (w *AsyncqWorker) Stop() {
	log.Info("Stopping asyncq worker server")
	w.server.Stop()
}

func (w *AsyncqWorker) Shutdown() {
	log.Info("Shutting down asyncq worker server")
	w.server.Shutdown()
}

func (w *AsyncqWorker) Run() error {
	log.Info("Running asyncq worker server")
	return w.server.Run(w.mux)
}
