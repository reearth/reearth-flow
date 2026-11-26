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

// dispatchPolicy is how long a Cloud Run debug job may wait for a free worker
// instance, and how often it asks. Cloud Run is the only arbiter of capacity:
// no counter is shared between API replicas, so there is nothing to keep in
// sync.
type dispatchPolicy struct {
	// MaxWait is how long attempts may keep being refused before the job fails.
	MaxWait     time.Duration
	BaseBackoff time.Duration
	MaxBackoff  time.Duration
}

var (
	runDispatchPolicy   = dispatchPolicy{MaxWait: 10 * time.Minute, BaseBackoff: 2 * time.Second, MaxBackoff: 20 * time.Second}
	probeDispatchPolicy = dispatchPolicy{MaxWait: 2 * time.Minute, BaseBackoff: 2 * time.Second, MaxBackoff: 20 * time.Second}
)

// backoff is the delay after the nth refused attempt (n >= 1). r is a uniform
// draw from [0, 1) that places the delay between BaseBackoff and a ceiling
// that doubles from twice BaseBackoff up to MaxBackoff, so replicas retrying
// together spread out.
func (p dispatchPolicy) backoff(n int, r float64) time.Duration {
	ceiling := 2 * p.BaseBackoff
	for k := 1; k < n && ceiling < p.MaxBackoff; k++ {
		ceiling *= 2
	}
	ceiling = min(ceiling, p.MaxBackoff)
	return p.BaseBackoff + time.Duration(r*float64(ceiling-p.BaseBackoff))
}

// attemptOutcome is what one attempt's answer means for the wait.
type attemptOutcome int

const (
	// attemptFinal is any answer a retry cannot improve: the worker took the
	// request, or it failed in a way that may follow work that started.
	attemptFinal attemptOutcome = iota
	// attemptBusy is a full pool. Nothing ran, so trying again cannot run the
	// job twice.
	attemptBusy
)

func classifyAttempt(err error) attemptOutcome {
	if errors.Is(err, gateway.ErrWorkerBusy) {
		return attemptBusy
	}
	return attemptFinal
}

var (
	// errPoolStayedFull ends a wait whose attempts were all refused. It is
	// returned joined with the last refusal, which wraps
	// gateway.ErrRunNotStarted.
	errPoolStayedFull = errors.New("the worker pool stayed full")
	// errFinishedWhileWaiting means the stored job reached a terminal state
	// while it waited, as when it is cancelled from any replica. Whoever moved
	// the job owns its terminal state.
	errFinishedWhileWaiting = errors.New("the job finished while waiting for a worker")
)

// dispatchWhenFree calls attempt until it returns anything other than a busy
// pool, the stored job is terminal, or p.MaxWait passes. The stored job is
// reloaded before every attempt, so a cancel on another replica is seen within
// one backoff. It never writes the job's status.
func (i *Job) dispatchWhenFree(
	ctx context.Context,
	jobID id.JobID,
	p dispatchPolicy,
	attempt func(context.Context) (gateway.JobStatus, error),
) (gateway.JobStatus, error) {
	deadline := i.clock().Add(p.MaxWait)
	for n := 1; ; n++ {
		if i.finishedWhileWaiting(ctx, jobID) {
			return gateway.JobStatusCancelled, errFinishedWhileWaiting
		}
		status, err := attempt(ctx)
		if classifyAttempt(err) == attemptFinal {
			return status, err
		}
		if !i.clock().Before(deadline) {
			return gateway.JobStatusFailed, fmt.Errorf("%w for %s: %w", errPoolStayedFull, p.MaxWait, err)
		}
		delay := p.backoff(n, rand.Float64())
		log.Infofc(ctx, "job: worker pool full for %s after attempt %d, retrying in %s", jobID, n, delay)
		if err := i.pause(ctx, delay); err != nil {
			return gateway.JobStatusFailed, err
		}
	}
}

// finishedWhileWaiting reports whether the stored job is gone or terminal. A
// read error does not end the wait: the next attempt reads again, and the wait
// is bounded by its deadline.
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

// isCloudRunDebug is the one definition of a job run on the Cloud Run worker:
// a debug job with no Batch job behind it.
func isCloudRunDebug(j *job.Job) bool {
	return j.Debug() != nil && *j.Debug() && j.GCPJobID() == ""
}

// dispatchLeaseKeyPrefix starts the key holding a job's dispatch lease. The
// goroutine dispatching a Cloud Run job renews it, so a job whose lease ran out
// lost the API instance that was dispatching it.
const dispatchLeaseKeyPrefix = "flow:cloud-run-dispatch-leases:"

const (
	dispatchLeaseLifetime = time.Minute
	dispatchLeaseRenewal  = dispatchLeaseLifetime / 3
	// abandonedDispatchAge is how old a PENDING job must be before a missing
	// lease is read as a lost dispatcher. It covers the gap between saving the
	// job and its dispatcher taking the lease.
	abandonedDispatchAge = 2 * time.Minute
	dispatcherMember     = "dispatcher"
)

func dispatchLeaseKey(jobID id.JobID) string {
	return dispatchLeaseKeyPrefix + jobID.String()
}

// holdDispatchLease keeps jobID's dispatch lease until the returned function
// is called. Without a usage counter there is no lease, and no job is ever
// read as abandoned.
func (i *Job) holdDispatchLease(ctx context.Context, jobID id.JobID) func() {
	if i.usage == nil {
		return func() {}
	}
	ctx, cancel := context.WithCancel(ctx)
	key := dispatchLeaseKey(jobID)
	renew := func() {
		now := i.clock()
		if _, err := i.usage.AcquireLease(ctx, key, dispatcherMember, now, now.Add(dispatchLeaseLifetime)); err != nil && ctx.Err() == nil {
			log.Warnfc(ctx, "job: could not renew the dispatch lease of %s: %v", jobID, err)
		}
	}
	renew()
	done := make(chan struct{})
	go func() {
		defer close(done)
		ticker := time.NewTicker(dispatchLeaseRenewal)
		defer ticker.Stop()
		for {
			select {
			case <-ctx.Done():
				return
			case <-ticker.C:
				renew()
			}
		}
	}()
	return func() {
		cancel()
		<-done
	}
}

// mayBeAbandoned is the cheap part of the abandoned-dispatch check, which
// needs no lock and no lease.
func (i *Job) mayBeAbandoned(j *job.Job) bool {
	return i.usage != nil && j.Status() == job.StatusPending && isCloudRunDebug(j) &&
		i.clock().Sub(j.StartedAt()) >= abandonedDispatchAge
}

// claimAbandonedDispatch reports whether j is a PENDING Cloud Run job whose
// dispatcher is gone, and claims failing it for the caller. The claim is a
// lease of its own under the job's key: it is the only lease there exactly when
// no dispatcher holds one and no other replica has claimed the job, so one
// replica wins. A claim that is not acted on runs out, so a later check can
// claim the job again.
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

// failAbandonedDispatch fails current when its dispatcher is gone, and reports
// whether it did. The caller holds the job's lock and passes the stored job.
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

// pause waits d, or until ctx is done.
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
