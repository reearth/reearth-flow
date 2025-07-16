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

// AsyncqWorker handles asyncq job processing
type AsyncqWorker struct {
	server       *asynq.Server
	mux          *asynq.ServeMux
	config       *Config
	jobRepo      repo.Job
	fileGateway  gateway.File
	batchGateway gateway.Batch
}

// NewAsyncqWorker creates a new asyncq worker
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

	// Register task handlers
	worker.registerHandlers()

	return worker
}

// registerHandlers registers task handlers with the mux
func (w *AsyncqWorker) registerHandlers() {
	w.mux.HandleFunc(TypeWorkflowJob, w.HandleWorkflowJob)
}

// HandleWorkflowJob handles workflow job execution
func (w *AsyncqWorker) HandleWorkflowJob(ctx context.Context, task *asynq.Task) error {
	payload, err := ParseWorkflowJobPayload(task)
	if err != nil {
		return fmt.Errorf("failed to parse workflow job payload: %w", err)
	}

	log.Infof("Processing workflow job: %s", payload.JobID)

	// Parse IDs
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

	// Get job from repository
	j, err := w.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		return fmt.Errorf("job not found: %w", err)
	}

	// Update job status to running
	j.SetStatus(job.StatusRunning)
	if err := w.jobRepo.Save(ctx, j); err != nil {
		log.Warnf("Failed to update job status to running: %v", err)
	}

	// Execute the workflow job
	err = w.executeWorkflow(ctx, payload, jobID, projectID, workspaceID, deploymentID)

	// Update job status based on execution result
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

// executeWorkflow executes the actual workflow
func (w *AsyncqWorker) executeWorkflow(
	ctx context.Context,
	payload WorkflowJobPayload,
	jobID id.JobID,
	projectID id.ProjectID,
	workspaceID accountdomain.WorkspaceID,
	deploymentID id.DeploymentID,
) error {
	// This is where we would call the actual workflow execution
	// For now, we'll simulate the workflow execution

	log.Infof("Starting workflow execution for job %s", jobID)
	log.Infof("Workflow URL: %s", payload.WorkflowURL)
	log.Infof("Metadata URL: %s", payload.MetadataURL)

	// Simulate workflow execution time
	// In a real implementation, this would:
	// 1. Download the workflow from the URL
	// 2. Parse the workflow definition
	// 3. Execute the workflow using the runtime engine
	// 4. Handle any errors or outputs

	// For demonstration, we'll just wait a bit
	time.Sleep(5 * time.Second)

	// Simulate success/failure based on debug flag
	if payload.Debug {
		log.Infof("Debug mode enabled, simulating successful execution")
		return nil
	}

	// In a real implementation, this would return the actual execution result
	log.Infof("Workflow execution completed for job %s", jobID)
	return nil
}

// updateJobArtifacts updates job artifacts after completion
func (w *AsyncqWorker) updateJobArtifacts(ctx context.Context, j *job.Job) error {
	jobID := j.ID().String()

	// Get output artifacts
	outputs, err := w.fileGateway.ListJobArtifacts(ctx, jobID)
	if err != nil {
		return fmt.Errorf("failed to list job artifacts: %w", err)
	}
	j.SetOutputURLs(outputs)

	// Get log URL
	logURL := w.fileGateway.GetJobLogURL(jobID)
	if logURL != "" {
		j.SetLogsURL(logURL)
	}

	// Get worker log URL
	workerLogURL := w.fileGateway.GetJobWorkerLogURL(jobID)
	if workerLogURL != "" {
		j.SetWorkerLogsURL(workerLogURL)
	}

	return nil
}

// Start starts the asyncq worker
func (w *AsyncqWorker) Start() error {
	log.Info("Starting asyncq worker server")
	return w.server.Start(w.mux)
}

// Stop stops the asyncq worker gracefully
func (w *AsyncqWorker) Stop() {
	log.Info("Stopping asyncq worker server")
	w.server.Stop()
}

// Shutdown shuts down the asyncq worker
func (w *AsyncqWorker) Shutdown() {
	log.Info("Shutting down asyncq worker server")
	w.server.Shutdown()
}

// Run starts the worker and blocks until shutdown
func (w *AsyncqWorker) Run() error {
	log.Info("Running asyncq worker server")
	return w.server.Run(w.mux)
}
