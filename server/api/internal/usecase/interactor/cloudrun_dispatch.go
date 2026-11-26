package interactor

import (
	"context"
	"errors"
	"fmt"
	"math/rand/v2"
	"time"

	"github.com/google/uuid"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/log"
)

type dispatchPolicy struct {
	MaxWait     time.Duration
	BaseBackoff time.Duration
	MaxBackoff  time.Duration
}

var (
	runDispatchPolicy   = dispatchPolicy{MaxWait: 10 * time.Minute, BaseBackoff: 2 * time.Second, MaxBackoff: 10 * time.Second}
	probeDispatchPolicy = dispatchPolicy{MaxWait: 2 * time.Minute, BaseBackoff: 2 * time.Second, MaxBackoff: 10 * time.Second}
)

func (p dispatchPolicy) backoff(n int, jitter float64) time.Duration {
	ceiling := 2 * p.BaseBackoff
	for k := 1; k < n && ceiling < p.MaxBackoff; k++ {
		ceiling *= 2
	}
	ceiling = min(ceiling, p.MaxBackoff)
	return p.BaseBackoff + time.Duration(jitter*float64(ceiling-p.BaseBackoff))
}

type attemptOutcome int

const (
	attemptMayHaveRun attemptOutcome = iota
	attemptNeverRan
)

func classifyAttempt(err error) attemptOutcome {
	if errors.Is(err, gateway.ErrWorkerBusy) {
		return attemptNeverRan
	}
	return attemptMayHaveRun
}

var (
	errPoolStayedFull       = errors.New("the worker pool stayed full")
	errFinishedWhileWaiting = errors.New("the job finished while waiting for a worker")
)

func (i *Job) dispatchWhenFree(
	ctx context.Context,
	jobID id.JobID,
	p dispatchPolicy,
	attempt func(context.Context) (gateway.JobStatus, error),
) (gateway.JobStatus, error) {
	release := i.holdDispatchLease(ctx, jobID)
	handedToWorker := false
	defer func() { release(handedToWorker) }()

	deadline := i.clock().Add(p.MaxWait)
	for n := 1; ; n++ {
		if i.finishedWhileWaiting(ctx, jobID) {
			return gateway.JobStatusCancelled, errFinishedWhileWaiting
		}
		status, err := attempt(ctx)
		if classifyAttempt(err) == attemptMayHaveRun {
			handedToWorker = true
			return status, err
		}
		now := i.clock()
		if !now.Before(deadline) {
			return gateway.JobStatusFailed, fmt.Errorf("%w for %s: %w (last refusal: %v)", errPoolStayedFull, p.MaxWait, gateway.ErrRunNotStarted, err)
		}
		delay := min(p.backoff(n, rand.Float64()), deadline.Sub(now))
		log.Infofc(ctx, "job: worker pool full for %s after attempt %d, retrying in %s", jobID, n, delay)
		if err := i.pause(ctx, delay); err != nil {
			return gateway.JobStatusFailed, err
		}
	}
}

func (i *Job) finishedWhileWaiting(ctx context.Context, jobID id.JobID) bool {
	j, err := i.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		log.Warnfc(ctx, "job: could not reload %s before dispatching it: %v", jobID, err)
		return false
	}
	return j == nil || isTerminal(j.Status())
}

func isTerminal(s job.Status) bool {
	return s == job.StatusCompleted || s == job.StatusFailed || s == job.StatusCancelled
}

func isCloudRunDebug(j *job.Job) bool {
	return j.Debug() != nil && *j.Debug() && j.GCPJobID() == ""
}

const dispatchLeaseKeyPrefix = "flow:cloud-run-dispatch-leases:"

const (
	dispatchLeaseLifetime     = 2 * time.Minute
	dispatchLeaseRenewal      = 20 * time.Second
	dispatchLeaseStartupGrace = 2 * time.Minute
	dispatchResultWait        = 15 * time.Minute
	dispatcherMember          = "dispatcher"
)

func dispatchLeaseKey(jobID id.JobID) string {
	return dispatchLeaseKeyPrefix + jobID.String()
}

// holdDispatchLease renews the job's dispatch lease until release is called.
// Once a worker may have the run, release(true) leaves the lease to live
// dispatchResultWait, so the job is not read as abandoned while its result is
// on the way.
func (i *Job) holdDispatchLease(ctx context.Context, jobID id.JobID) (release func(handedToWorker bool)) {
	if i.usage == nil {
		return func(bool) {}
	}
	renewCtx, stopRenewing := context.WithCancel(ctx)
	key := dispatchLeaseKey(jobID)
	hold := func(ctx context.Context, d time.Duration) {
		now := i.clock()
		if _, err := i.usage.AcquireLease(ctx, key, dispatcherMember, now, now.Add(d)); err != nil && ctx.Err() == nil {
			log.Warnfc(ctx, "job: could not renew the dispatch lease of %s: %v", jobID, err)
		}
	}
	hold(renewCtx, dispatchLeaseLifetime)
	done := make(chan struct{})
	go func() {
		defer close(done)
		ticker := time.NewTicker(dispatchLeaseRenewal)
		defer ticker.Stop()
		for {
			select {
			case <-renewCtx.Done():
				return
			case <-ticker.C:
				hold(renewCtx, dispatchLeaseLifetime)
			}
		}
	}()
	return func(handedToWorker bool) {
		stopRenewing()
		<-done
		if handedToWorker {
			hold(context.WithoutCancel(ctx), dispatchResultWait)
		}
	}
}

func (i *Job) mayBeAbandoned(j *job.Job) bool {
	return i.usage != nil && j.Status() == job.StatusPending && isCloudRunDebug(j) &&
		i.clock().Sub(j.StartedAt()) >= dispatchLeaseStartupGrace
}

func (i *Job) claimAbandonedDispatch(ctx context.Context, j *job.Job) bool {
	if !i.mayBeAbandoned(j) {
		return false
	}
	key := dispatchLeaseKey(j.ID())
	claim := uuid.NewString()
	now := i.clock()
	n, err := i.usage.AcquireLease(ctx, key, claim, now, now.Add(dispatchLeaseLifetime))
	if err != nil {
		log.Warnfc(ctx, "job: could not check the dispatch lease of %s: %v", j.ID(), err)
		return false
	}
	if n == 1 {
		return true
	}
	if err := i.usage.ReleaseLease(context.WithoutCancel(ctx), key, claim); err != nil {
		log.Warnfc(ctx, "job: could not drop a dispatch check of %s: %v", j.ID(), err)
	}
	return false
}

// The worker stamps its logs with its own clock, so the window has no upper
// bound the server could trust.
var logsUnbounded = time.Date(9999, time.December, 31, 0, 0, 0, 0, time.UTC)

// startRunningOnFirstLog reports whether the queued job's worker has logged,
// and if so sets the job RUNNING unless it moved on in the meantime.
func (i *Job) startRunningOnFirstLog(ctx context.Context, current *job.Job) (logged bool, err error) {
	if current.Status() != job.StatusPending || !isCloudRunDebug(current) {
		return false, nil
	}
	logs, err := i.redis.GetLogs(ctx, time.Time{}, logsUnbounded, current.ID())
	if err != nil {
		log.Warnfc(ctx, "job: could not read the logs of queued cloud run job %s: %v", current.ID(), err)
		return false, nil
	}
	if len(logs) == 0 {
		return false, nil
	}
	latest, err := i.jobRepo.FindByID(ctx, current.ID())
	if err != nil {
		return true, fmt.Errorf("failed to reload started cloud run job %s: %w", current.ID(), err)
	}
	if latest == nil || latest.Status() != job.StatusPending {
		return true, nil
	}
	latest.SetStatus(job.StatusRunning)
	if err := i.jobRepo.Save(ctx, latest); err != nil {
		return true, fmt.Errorf("failed to save started cloud run job %s: %w", current.ID(), err)
	}
	i.subscriptions.Notify(latest.ID().String(), latest.Status())
	return true, nil
}

func (i *Job) failAbandonedDispatch(ctx context.Context, current *job.Job) bool {
	if !i.claimAbandonedDispatch(ctx, current) {
		return false
	}
	log.Warnfc(ctx, "job: %s is still queued but nothing is dispatching it, failing it", current.ID())
	current.SetStatus(job.StatusFailed)
	if err := i.jobRepo.Save(ctx, current); err != nil {
		log.Errorfc(ctx, "job: failed to save abandoned cloud run job %s: %v", current.ID(), err)
		return false
	}
	if err := i.handleJobCompletion(ctx, current); err != nil {
		log.Errorfc(ctx, "job: completion handling failed for abandoned cloud run job %s: %v", current.ID(), err)
	}
	i.subscriptions.Notify(current.ID().String(), current.Status())
	return true
}

func (i *Job) clock() time.Time {
	if i.now == nil {
		return time.Now()
	}
	return i.now()
}

func (i *Job) pause(ctx context.Context, d time.Duration) error {
	if i.sleep != nil {
		return i.sleep(ctx, d)
	}
	t := time.NewTimer(d)
	defer t.Stop()
	select {
	case <-ctx.Done():
		return ctx.Err()
	case <-t.C:
		return nil
	}
}
