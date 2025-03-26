package interactor

import (
	"context"
	"fmt"
	"time"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/notification"
	"github.com/reearth/reearthx/log"
)

func (i *Job) handleJobCompletion(ctx context.Context, j *job.Job) error {
	if j == nil {
		return fmt.Errorf("job cannot be nil")
	}

	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return err
	}

	jobID := j.ID().String()
	log.Infof("handling job completion for ID %s with status %s", jobID, j.Status())

	config := i.monitor.Get(jobID)

	var artifactErr error
	for retries := 0; retries < 3; retries++ {
		if retries > 0 {
			log.Infof("retry %d: updating artifacts for job ID %s", retries, jobID)
			time.Sleep(2 * time.Second)
		}

		artifactErr = i.updateJobArtifacts(ctx, j)
		if artifactErr == nil && len(j.OutputURLs()) > 0 {
			break
		}
	}

	if artifactErr != nil {
		log.Errorfc(ctx, "job: failed to update artifacts after retries for jobID=%s: %v", jobID, artifactErr)
	}

	if err := i.saveJobState(ctx, j); err != nil {
		return fmt.Errorf("failed to save job state: %w", err)
	}

	if config == nil || config.NotificationURL == nil || *config.NotificationURL == "" {
		return nil
	}

	if err := i.sendCompletionNotification(ctx, j, *config.NotificationURL); err != nil {
		return fmt.Errorf("failed to send notification: %w", err)
	}

	return nil
}

func (i *Job) sendCompletionNotification(ctx context.Context, j *job.Job, notificationURL string) error {
	jobID := j.ID().String()

	status := "failed"
	switch j.Status() {
	case job.StatusCompleted:
		status = "succeeded"
	case job.StatusCancelled:
		status = "cancelled"
	}

	var logs []string
	logExists, err := i.file.CheckJobLogExists(ctx, jobID)
	if err != nil {
		log.Warnfc(ctx, "job: failed to check log existence for jobID=%s: %v", jobID, err)
	} else if logExists {
		logs = append(logs, j.LogsURL())
	}

	payload := notification.Payload{
		RunID:        jobID,
		DeploymentID: j.Deployment().String(),
		Status:       status,
		Logs:         logs,
		Outputs:      j.OutputURLs(),
	}

	return i.notifier.Send(notificationURL, payload)
}
