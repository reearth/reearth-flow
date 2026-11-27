package interactor

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/notification"
	"github.com/reearth/reearthx/log"
)

// Reconcile syncs a job with Batch and its completion event and finalizes it once, from any instance.
func (i *Job) Reconcile(ctx context.Context, jobID id.JobID) (*job.Job, error) {
	lock := i.getJobLock(jobID.String())
	lock.Lock()
	defer lock.Unlock()

	return i.reconcileLocked(ctx, jobID)
}

// reconcileLocked is Reconcile for a caller that holds the job's lock.
func (i *Job) reconcileLocked(ctx context.Context, jobID id.JobID) (*job.Job, error) {
	j, err := i.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		return nil, err
	}
	if j.Status().IsTerminal() {
		// Another call, possibly on another instance, finished it.
		i.catchUpFinished(ctx, j)
		i.subscriptions.Notify(jobID.String(), j.Status())
		return j, nil
	}

	changed := false

	// Cloud Run debug jobs have no Batch job to ask.
	if j.GCPJobID() != "" && i.batch != nil {
		s, err := i.batch.GetJobStatus(ctx, j.GCPJobID())
		if err != nil {
			return nil, err
		}
		if bs := job.Status(s); j.BatchStatus() == nil || *j.BatchStatus() != bs {
			j.SetBatchStatus(bs)
			changed = true
		}
	}

	var event *gateway.JobCompleteEvent
	var eventErr error
	if i.redis != nil {
		event, eventErr = i.redis.GetJobCompleteEvent(ctx, jobID)
		if eventErr != nil {
			log.Warnfc(ctx, "job: failed to get the completion event for job %s: %v", jobID, eventErr)
		}
	}
	if ws := workerStatusOf(event); ws != "" && (j.WorkerStatus() == nil || *j.WorkerStatus() != ws) {
		j.SetWorkerStatus(ws)
		changed = true
	}

	if j.Status().IsTerminal() {
		got, _, err := i.finalize(ctx, j, event)
		return got, err
	}
	// A queued Cloud Run job with no result yet runs from its first log, or fails if nothing dispatches it.
	if event == nil && eventErr == nil && !changed && i.redis != nil {
		logged, err := i.startRunningOnFirstLog(ctx, j)
		if err != nil {
			return nil, err
		}
		if logged || i.failAbandonedDispatch(ctx, j) {
			return i.jobRepo.FindByID(ctx, jobID)
		}
	}
	if !changed {
		return j, nil
	}

	// Still running: keep the event for its finish time and diagnostics.
	j.SetStatus(j.Status())
	saved, err := i.jobRepo.SaveIfNotTerminal(ctx, j)
	if err != nil {
		return nil, err
	}
	if !saved {
		return i.reloadFinished(ctx, jobID)
	}
	i.subscriptions.Notify(jobID.String(), j.Status())
	return j, nil
}

// finalize saves terminal j unless already finished elsewhere; only the winner runs side effects.
func (i *Job) finalize(ctx context.Context, j *job.Job, event *gateway.JobCompleteEvent) (*job.Job, bool, error) {
	if j.CompletedAt() == nil {
		finishedAt := time.Now()
		if event != nil && !event.Timestamp.IsZero() {
			finishedAt = event.Timestamp
		}
		j.SetCompletedAt(&finishedAt)
	}
	j.SetStatus(j.Status())

	// Before the event is deleted, or a failure loses diagnostics; writes are idempotent.
	diagnosticsPersisted := true
	if event != nil {
		if err := i.persistTerminalDiagnostics(ctx, j.ID(), event); err != nil {
			diagnosticsPersisted = false
			log.Warnfc(ctx, "job: failed to persist terminal diagnostics for job %s: %v", j.ID(), err)
		}
	}

	if err := i.updateJobArtifacts(ctx, j); err != nil {
		log.Errorfc(ctx, "job: failed to update artifacts for job %s: %v", j.ID(), err)
	}

	saved, err := i.jobRepo.SaveIfNotTerminal(ctx, j)
	if err != nil {
		return nil, false, err
	}
	if !saved {
		got, err := i.reloadFinished(ctx, j.ID())
		return got, false, err
	}

	if event != nil && diagnosticsPersisted && i.redis != nil {
		if err := i.redis.DeleteJobCompleteEvent(ctx, j.ID()); err != nil {
			log.Warnfc(ctx, "job: failed to delete the completion event for job %s: %v", j.ID(), err)
		}
	}
	logJobFinished(ctx, j)
	i.deliverNotification(ctx, j)
	i.subscriptions.Notify(j.ID().String(), j.Status())
	return j, true, nil
}

// catchUpFinished retries what finishing a job left undone: diagnostics, artifact listing and the webhook.
func (i *Job) catchUpFinished(ctx context.Context, j *job.Job) {
	if i.redis != nil {
		event, err := i.redis.GetJobCompleteEvent(ctx, j.ID())
		if err != nil {
			log.Warnfc(ctx, "job: failed to get the completion event for finished job %s: %v", j.ID(), err)
		} else if event != nil {
			if err := i.persistTerminalDiagnostics(ctx, j.ID(), event); err != nil {
				log.Warnfc(ctx, "job: failed to persist terminal diagnostics for job %s: %v", j.ID(), err)
			} else if err := i.redis.DeleteJobCompleteEvent(ctx, j.ID()); err != nil {
				log.Warnfc(ctx, "job: failed to delete the completion event for job %s: %v", j.ID(), err)
			}
		}
	}

	if j.ArtifactsPending() {
		if err := i.updateJobArtifacts(ctx, j); err != nil {
			log.Errorfc(ctx, "job: failed to update artifacts for job %s: %v", j.ID(), err)
		} else if err := i.jobRepo.Save(ctx, j); err != nil {
			log.Errorfc(ctx, "job: failed to save artifacts for job %s: %v", j.ID(), err)
		}
	}

	i.deliverNotification(ctx, j)
}

// reloadFinished returns the job another writer finished and notifies subscribers.
func (i *Job) reloadFinished(ctx context.Context, jobID id.JobID) (*job.Job, error) {
	j, err := i.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		return nil, err
	}
	i.subscriptions.Notify(jobID.String(), j.Status())
	return j, nil
}

const (
	// notificationPreflightTimeout bounds the log-existence checks before a webhook is sent.
	notificationPreflightTimeout = 20 * time.Second
	// notificationClaimLease outlasts the preflight checks plus the notifier's own timeout.
	notificationClaimLease = notificationPreflightTimeout + notification.SendTimeout + 10*time.Second
)

// deliverNotification sends an unsent webhook once outputs are listed; a claim keeps instances from sending it twice.
func (i *Job) deliverNotification(ctx context.Context, j *job.Job) {
	url := j.NotificationURL()
	if url == nil || j.NotifiedAt() != nil || j.ArtifactsPending() {
		return
	}
	now := time.Now()
	claimed, err := i.jobRepo.ClaimNotification(ctx, j.ID(), now, now.Add(notificationClaimLease))
	if err != nil {
		log.Errorfc(ctx, "job: failed to claim the completion webhook for job %s: %v", j.ID(), err)
		return
	}
	if !claimed {
		return
	}
	preflightCtx, cancel := context.WithTimeout(ctx, notificationPreflightTimeout)
	defer cancel()
	if err := i.sendCompletionNotification(preflightCtx, j, *url); err != nil {
		// The claim lapses, so a later reconcile retries.
		log.Errorfc(ctx, "job: failed to send the completion webhook for job %s: %v", j.ID(), err)
		return
	}
	sentAt := time.Now()
	j.SetNotifiedAt(&sentAt)
	if err := i.jobRepo.MarkNotified(ctx, j.ID(), sentAt); err != nil {
		log.Errorfc(ctx, "job: sent the completion webhook for job %s but could not record it: %v", j.ID(), err)
	}
}

// workerStatusOf is the status a completion event reports, or "" for none.
func workerStatusOf(event *gateway.JobCompleteEvent) job.Status {
	if event == nil {
		return ""
	}
	switch event.Result {
	case "success":
		return job.StatusCompleted
	case "failed":
		return job.StatusFailed
	}
	return ""
}
