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

type AsyncqBatch struct {
	client     *asynq.Client
	inspector  *asynq.Inspector
	config     *Config
	jobTracker *jobTracker
}

type jobTracker struct {
	jobs map[string]*JobStatus
	mu   sync.RWMutex
}

type JobStatus struct {
	ID          string
	TaskID      string
	Status      gateway.JobStatus
	CreatedAt   time.Time
	UpdatedAt   time.Time
	CompletedAt *time.Time
	Error       error
}

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

func (b *AsyncqBatch) SubmitJob(
	ctx context.Context,
	jobID id.JobID,
	workflowURL, metadataURL string,
	variables map[string]interface{},
	projectID id.ProjectID,
	workspaceID accountdomain.WorkspaceID,
) (string, error) {
	deploymentID := id.NewDeploymentID()

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

	opts := []asynq.Option{
		asynq.MaxRetry(b.config.MaxRetry),
		asynq.Queue("default"),
		asynq.Timeout(30 * time.Minute),
		asynq.Unique(24 * time.Hour),
	}

	info, err := b.client.EnqueueContext(ctx, task, opts...)
	if err != nil {
		return "", fmt.Errorf("failed to enqueue task: %w", err)
	}

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

func (b *AsyncqBatch) GetJobStatus(ctx context.Context, jobName string) (gateway.JobStatus, error) {
	b.jobTracker.mu.RLock()
	if jobStatus, exists := b.jobTracker.jobs[jobName]; exists {
		b.jobTracker.mu.RUnlock()

		if updatedStatus, err := b.getTaskStatus(ctx, jobStatus.TaskID); err == nil {
			b.updateJobStatus(jobName, updatedStatus)
			return updatedStatus, nil
		}

		return jobStatus.Status, nil
	}
	b.jobTracker.mu.RUnlock()

	return b.getTaskStatus(ctx, jobName)
}

func (b *AsyncqBatch) getTaskStatus(ctx context.Context, taskID string) (gateway.JobStatus, error) {
	pendingTasks, err := b.inspector.ListPendingTasks("default")
	if err == nil {
		for _, task := range pendingTasks {
			if task.ID == taskID {
				return gateway.JobStatusPending, nil
			}
		}
	}

	activeTasks, err := b.inspector.ListActiveTasks("default")
	if err == nil {
		for _, task := range activeTasks {
			if task.ID == taskID {
				return gateway.JobStatusRunning, nil
			}
		}
	}

	completedTasks, err := b.inspector.ListCompletedTasks("default")
	if err == nil {
		for _, task := range completedTasks {
			if task.ID == taskID {
				return gateway.JobStatusCompleted, nil
			}
		}
	}

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

func (b *AsyncqBatch) CancelJob(ctx context.Context, jobName string) error {
	b.jobTracker.mu.Lock()
	defer b.jobTracker.mu.Unlock()

	jobStatus, exists := b.jobTracker.jobs[jobName]
	if !exists {
		return fmt.Errorf("job not found: %s", jobName)
	}

	err := b.inspector.DeleteTask("default", jobStatus.TaskID)
	if err != nil {
		log.Warnf("Failed to delete task %s: %v", jobStatus.TaskID, err)
	}

	jobStatus.Status = gateway.JobStatusCancelled
	jobStatus.UpdatedAt = time.Now()
	now := time.Now()
	jobStatus.CompletedAt = &now

	log.Infof("Job %s cancelled", jobName)

	return nil
}

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

func (b *AsyncqBatch) GetJobTracker() *jobTracker {
	return b.jobTracker
}

func (b *AsyncqBatch) SetJobStatus(jobName string, status gateway.JobStatus) {
	b.updateJobStatus(jobName, status)
}
