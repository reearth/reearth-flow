package interactor

import (
	"context"
	"fmt"
	"log"
	"strings"
	"time"

	"github.com/reearth/reearth-flow/subscriber/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/subscriber/pkg/job"
)

type JobSubscriberUseCase interface {
	ProcessJobStatusEvent(ctx context.Context, event *job.JobStatusEvent) error
}

type jobSubscriberUseCase struct {
	storage    gateway.JobStorage
	apiGateway gateway.APIGateway
}

func NewJobSubscriberUseCase(storage gateway.JobStorage, apiGateway gateway.APIGateway) JobSubscriberUseCase {
	return &jobSubscriberUseCase{
		storage:    storage,
		apiGateway: apiGateway,
	}
}

func (u *jobSubscriberUseCase) ProcessJobStatusEvent(ctx context.Context, event *job.JobStatusEvent) error {
	if event == nil {
		log.Printf("ERROR: Received nil job status event")
		return fmt.Errorf("event is nil")
	}

	log.Printf("DEBUG: Processing job status event for JobID: %s with status %s",
		event.JobID, event.Status)

	upperStatus := strings.ToUpper(string(event.Status))
	event.Status = job.Status(upperStatus)

	if err := u.storage.SaveToRedis(ctx, event); err != nil {
		log.Printf("ERROR: Failed to save job status event to Redis for JobID %s: %v", event.JobID, err)
		return fmt.Errorf("failed to write to Redis: %w", err)
	}
	log.Printf("DEBUG: Successfully saved job status event to Redis for JobID: %s", event.JobID)

	if event.Status == job.StatusCompleted || event.Status == job.StatusFailed || event.Status == job.StatusCancelled {
		jobRecord := &job.Job{
			ID:         event.JobID,
			WorkflowID: event.WorkflowID,
			Status:     event.Status,
			Message:    event.Message,
			UpdatedAt:  time.Now(),
		}

		if event.FailedNodes != nil {
			jobRecord.FailedNodes = *event.FailedNodes
		}

		now := time.Now()
		jobRecord.CompletedAt = &now
		log.Printf("DEBUG: Setting CompletedAt=%s for job %s", now.Format(time.RFC3339), event.JobID)

		if err := u.storage.SaveToMongo(ctx, event.JobID, jobRecord); err != nil {
			log.Printf("WARNING: Failed to save job to MongoDB for JobID=%s: %v",
				event.JobID, err)
		} else {
			log.Printf("DEBUG: Successfully saved job to MongoDB for JobID=%s", event.JobID)
		}

		if err := u.apiGateway.NotifyJobStatusChange(ctx, event.JobID, string(event.Status)); err != nil {
			log.Printf("WARNING: Failed to notify API server about job status change for JobID=%s: %v",
				event.JobID, err)
		} else {
			log.Printf("DEBUG: Successfully notified API server about job status change for JobID=%s", event.JobID)
		}
	} else if event.Status == job.StatusRunning && event.Status != job.StatusStarting {
		jobRecord := &job.Job{
			ID:         event.JobID,
			WorkflowID: event.WorkflowID,
			Status:     event.Status,
			Message:    event.Message,
			UpdatedAt:  time.Now(),
		}

		now := time.Now()
		jobRecord.StartedAt = &now

		if err := u.storage.SaveToMongo(ctx, event.JobID, jobRecord); err != nil {
			log.Printf("WARNING: Failed to save job to MongoDB for JobID=%s: %v",
				event.JobID, err)
		}

		log.Printf("DEBUG: Skipping API notification for non-terminal status %s", event.Status)
	}

	log.Printf("DEBUG: Successfully processed job status event for JobID: %s", event.JobID)
	return nil
}
