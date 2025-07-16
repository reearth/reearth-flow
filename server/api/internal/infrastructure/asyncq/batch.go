package asyncq

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/hibiken/asynq"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/log"
)

// AsyncqBatch implements gateway.Batch interface using asyncq
type AsyncqBatch struct {
	client     *asynq.Client
	inspector  *asynq.Inspector
	config     *Config
	jobTracker *jobTracker
	mu         sync.RWMutex
}

// jobTracker tracks job statuses
type jobTracker struct {
	jobs map[string]*JobStatus
	mu   sync.RWMutex
}

// JobStatus represents the status of a job
type JobStatus struct {
	ID          string
	TaskID      string
	Status      gateway.JobStatus
	CreatedAt   time.Time
	UpdatedAt   time.Time
	CompletedAt *time.Time
	Error       error
}

// NewAsyncqBatch creates a new asyncq batch implementation
func NewAsyncqBatch(config *Config) (*AsyncqBatch, error) {
	redisOpt := config.GetRedisClientOpt()

	client := asynq.NewClient(redisOpt)
	inspector := asynq.NewInspector(redisOpt)

	return &AsyncqBatch{
		client:     client,
		inspector:  inspector,
		config:     config,
		jobTracker: &jobTracker{jobs: make(map[string]*JobStatus)},
	}, nil
}

// SubmitJob submits a job to the asyncq queue
func (b *AsyncqBatch) SubmitJob(
	ctx context.Context,
	jobID id.JobID,
	workflowURL, metadataURL string,
	variables map[string]interface{},
	projectID id.ProjectID,
	workspaceID accountdomain.WorkspaceID,
) (string, error) {
	// Create deployment ID if not provided
	deploymentID := id.NewDeploymentID()

	// Create task
	task, err := NewWorkflowJobTask(
		jobID,
		workflowURL,
		metadataURL,
		variables,
		projectID,
		workspaceID,
		deploymentID,
		nil,   // notificationURL
		false, // debug
	)
	if err != nil {
		return "", fmt.Errorf("failed to create task: %w", err)
	}

	// Configure task options
	opts := []asynq.Option{
		asynq.MaxRetry(b.config.MaxRetry),
		asynq.Queue("default"),
		asynq.Timeout(30 * time.Minute),
		asynq.Unique(24 * time.Hour), // Prevent duplicate jobs
	}

	// Enqueue task
	info, err := b.client.EnqueueContext(ctx, task, opts...)
	if err != nil {
		return "", fmt.Errorf("failed to enqueue task: %w", err)
	}

	// Track job status
	b.jobTracker.mu.Lock()
	jobKey := jobID.String()
	b.jobTracker.jobs[jobKey] = &JobStatus{
		ID:        jobKey,
		TaskID:    info.ID,
		Status:    gateway.JobStatusPending,
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
	}
	b.jobTracker.mu.Unlock()

	log.Infof("Job %s submitted to asyncq queue with task ID: %s", jobID, info.ID)

	return info.ID, nil
}

// GetJobStatus retrieves the status of a job
func (b *AsyncqBatch) GetJobStatus(ctx context.Context, jobName string) (gateway.JobStatus, error) {
	// Try to get from job tracker first
	b.jobTracker.mu.RLock()
	if jobStatus, exists := b.jobTracker.jobs[jobName]; exists {
		b.jobTracker.mu.RUnlock()

		// Try to get updated status from asyncq
		if updatedStatus, err := b.getTaskStatus(ctx, jobStatus.TaskID); err == nil {
			b.updateJobStatus(jobName, updatedStatus)
			return updatedStatus, nil
		}

		return jobStatus.Status, nil
	}
	b.jobTracker.mu.RUnlock()

	// If not found in tracker, try to find by task ID
	return b.getTaskStatus(ctx, jobName)
}

// getTaskStatus gets task status from asyncq inspector
func (b *AsyncqBatch) getTaskStatus(ctx context.Context, taskID string) (gateway.JobStatus, error) {
	// Check pending tasks
	pendingTasks, err := b.inspector.ListPendingTasks("default")
	if err == nil {
		for _, task := range pendingTasks {
			if task.ID == taskID {
				return gateway.JobStatusPending, nil
			}
		}
	}

	// Check active tasks
	activeTasks, err := b.inspector.ListActiveTasks("default")
	if err == nil {
		for _, task := range activeTasks {
			if task.ID == taskID {
				return gateway.JobStatusRunning, nil
			}
		}
	}

	// Check completed tasks
	completedTasks, err := b.inspector.ListCompletedTasks("default")
	if err == nil {
		for _, task := range completedTasks {
			if task.ID == taskID {
				return gateway.JobStatusCompleted, nil
			}
		}
	}

	// Check retry tasks (not failed tasks)
	retryTasks, err := b.inspector.ListRetryTasks("default")
	if err == nil {
		for _, task := range retryTasks {
			if task.ID == taskID {
				return gateway.JobStatusFailed, nil
			}
		}
	}

	return gateway.JobStatusUnknown, fmt.Errorf("task not found: %s", taskID)
}

// updateJobStatus updates the job status in tracker
func (b *AsyncqBatch) updateJobStatus(jobName string, status gateway.JobStatus) {
	b.jobTracker.mu.Lock()
	defer b.jobTracker.mu.Unlock()

	if jobStatus, exists := b.jobTracker.jobs[jobName]; exists {
		jobStatus.Status = status
		jobStatus.UpdatedAt = time.Now()

		if status == gateway.JobStatusCompleted || status == gateway.JobStatusFailed || status == gateway.JobStatusCancelled {
			now := time.Now()
			jobStatus.CompletedAt = &now
		}
	}
}

// ListJobs lists jobs for a project
func (b *AsyncqBatch) ListJobs(ctx context.Context, projectID id.ProjectID) ([]gateway.JobInfo, error) {
	var jobs []gateway.JobInfo

	b.jobTracker.mu.RLock()
	defer b.jobTracker.mu.RUnlock()

	for _, jobStatus := range b.jobTracker.jobs {
		jobIDParsed, err := id.JobIDFrom(jobStatus.ID)
		if err != nil {
			continue
		}

		jobs = append(jobs, gateway.JobInfo{
			ID:     jobIDParsed,
			Name:   jobStatus.ID,
			Status: jobStatus.Status,
		})
	}

	return jobs, nil
}

// CancelJob cancels a job
func (b *AsyncqBatch) CancelJob(ctx context.Context, jobName string) error {
	b.jobTracker.mu.Lock()
	defer b.jobTracker.mu.Unlock()

	jobStatus, exists := b.jobTracker.jobs[jobName]
	if !exists {
		return fmt.Errorf("job not found: %s", jobName)
	}

	// Try to delete the task instead of cancel
	err := b.inspector.DeleteTask("default", jobStatus.TaskID)
	if err != nil {
		log.Warnf("Failed to delete task %s: %v", jobStatus.TaskID, err)
		// Don't return error, just update status
	}

	// Update job status
	jobStatus.Status = gateway.JobStatusCancelled
	jobStatus.UpdatedAt = time.Now()
	now := time.Now()
	jobStatus.CompletedAt = &now

	log.Infof("Job %s cancelled", jobName)

	return nil
}

// Close closes the asyncq batch client
func (b *AsyncqBatch) Close() error {
	err1 := b.client.Close()
	err2 := b.inspector.Close()

	if err1 != nil {
		return err1
	}
	if err2 != nil {
		return err2
	}

	return nil
}

// GetJobTracker returns the job tracker for testing
func (b *AsyncqBatch) GetJobTracker() *jobTracker {
	return b.jobTracker
}

// SetJobStatus sets job status manually (for testing)
func (b *AsyncqBatch) SetJobStatus(jobName string, status gateway.JobStatus) {
	b.updateJobStatus(jobName, status)
}
