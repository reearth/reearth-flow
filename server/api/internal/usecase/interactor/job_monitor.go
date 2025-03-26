package interactor

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/job/monitor"
	"github.com/reearth/reearthx/log"
)

func (i *Job) StartMonitoring(ctx context.Context, j *job.Job, notificationURL *string) error {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return err
	}

	jobID := j.ID().String()

	if j.Status() == job.StatusCompleted || j.Status() == job.StatusFailed || j.Status() == job.StatusCancelled {
		return nil
	}

	monitorCtx, cancel := context.WithCancel(ctx)
	i.monitor.Register(jobID, &monitor.Config{
		Cancel:          cancel,
		NotificationURL: notificationURL,
	})

	i.monitoringMu.Lock()
	i.monitoringJobs[jobID] = time.Now()
	i.monitoringMu.Unlock()

	go i.runMonitoringLoop(monitorCtx, j)

	return nil
}

func (i *Job) runMonitoringLoop(ctx context.Context, j *job.Job) {
	ticker := time.NewTicker(10 * time.Second)
	defer ticker.Stop()

	jobID := j.ID().String()
	log.Infof("Beginning continuous monitoring for job %s", jobID)

	maxDuration := 24 * time.Hour
	startTime := time.Now()

	// Create our own subscription to detect terminal states
	// This ensures we get notified when the job completes, even if the status
	// is updated by another process
	statusCh := i.subscriptions.Subscribe(jobID)
	defer func() {
		log.Debugf("Unsubscribing monitor from job %s", jobID)
		i.subscriptions.Unsubscribe(j.ID().String(), statusCh)
	}()

	for {
		select {
		case <-ctx.Done():
			log.Infof("Context cancelled for job %s monitoring", jobID)
			return

		case status, ok := <-statusCh:
			if !ok {
				log.Infof("Status channel closed for job %s", jobID)
				return
			}

			if status == job.StatusCompleted || status == job.StatusFailed || status == job.StatusCancelled {
				log.Infof("Job %s reached terminal state %s via subscription notification, stopping monitoring",
					jobID, status)
				i.cleanupMonitoring(jobID)
				return
			}

		case <-ticker.C:
			if time.Since(startTime) > maxDuration {
				log.Warnf("Exceeded maximum monitoring duration (%s) for job %s", maxDuration, jobID)
				i.cleanupMonitoring(jobID)
				return
			}

			currentJob, err := i.jobRepo.FindByID(context.Background(), j.ID())
			if err != nil {
				log.Errorf("Failed to fetch current job state: %v", err)
				continue
			}

			if currentJob.Status() == job.StatusCompleted ||
				currentJob.Status() == job.StatusFailed ||
				currentJob.Status() == job.StatusCancelled {
				log.Infof("Job %s already in terminal state %s in database, stopping monitoring",
					jobID, currentJob.Status())
				i.cleanupMonitoring(jobID)
				return
			}

			if err := i.checkJobStatus(ctx, currentJob); err != nil {
				log.Errorf("Status check failed for job %s: %v", jobID, err)
				continue
			}
		}
	}
}

func (i *Job) cleanupMonitoring(jobID string) {
	log.Infof("Cleaning up monitoring resources for job %s", jobID)

	i.monitor.Remove(jobID)

	i.monitoringMu.Lock()
	delete(i.monitoringJobs, jobID)
	i.monitoringMu.Unlock()

	// Note: We don't clean up other subscribers here
	// Each subscriber (GraphQL clients, etc.) is responsible for its own cleanup
}
