package interactor

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/subscription"
	"github.com/reearth/reearth-flow/api/pkg/userfacinglog"
	reearth_log "github.com/reearth/reearthx/log"
)

type UserFacingLogInteractor struct {
	logsGatewayRedis  gateway.Redis
	jobRepo           repo.Job
	subscriptions     *subscription.UserFacingLogManager
	watchers          map[string]context.CancelFunc
	mu                sync.Mutex
	permissionChecker gateway.PermissionChecker
}

func NewUserFacingLogInteractor(lgRedis gateway.Redis, jobRepo repo.Job, permissionChecker gateway.PermissionChecker) interfaces.UserFacingLog {
	return &UserFacingLogInteractor{
		logsGatewayRedis:  lgRedis,
		jobRepo:           jobRepo,
		subscriptions:     subscription.NewUserFacingLogManager(),
		watchers:          make(map[string]context.CancelFunc),
		permissionChecker: permissionChecker,
	}
}

func (li *UserFacingLogInteractor) checkPermission(ctx context.Context, action string) error {
	return checkPermission(ctx, li.permissionChecker, rbac.ResourceLog, action)
}

func (li *UserFacingLogInteractor) GetUserFacingLogs(ctx context.Context, since time.Time, jobID id.JobID) ([]*userfacinglog.UserFacingLog, error) {
	if err := li.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	ctx, cancel := context.WithTimeout(ctx, 30*time.Second)
	defer cancel()
	until := time.Now().UTC()
	if li.logsGatewayRedis == nil {
		reearth_log.Error("logsGatewayRedis is nil: unable to get user-facing logs from Redis")
		return nil, fmt.Errorf("logsGatewayRedis is nil: unable to get user-facing logs from Redis")
	}
	logs, err := li.logsGatewayRedis.GetUserFacingLogs(ctx, since, until, jobID)
	if err != nil {
		return nil, fmt.Errorf("failed to get user-facing logs from Redis: %w", err)
	}
	return logs, nil
}

func (li *UserFacingLogInteractor) Subscribe(ctx context.Context, jobID id.JobID) (chan *userfacinglog.UserFacingLog, error) {
	if err := li.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	if li.logsGatewayRedis == nil {
		return nil, fmt.Errorf("logsGatewayRedis is nil")
	}

	ch := li.subscriptions.Subscribe(jobID.String())
	since := time.Now().Add(-10 * time.Minute).UTC()

	go func() {
		initialLogs, err := li.logsGatewayRedis.GetUserFacingLogs(ctx, since, time.Now().UTC(), jobID)
		if err == nil && len(initialLogs) > 0 {
			li.subscriptions.Notify(jobID.String(), initialLogs)
		}
	}()

	li.startWatchingLogsIfNeeded(jobID, since)

	return ch, nil
}

func (li *UserFacingLogInteractor) startWatchingLogsIfNeeded(jobID id.JobID, since time.Time) {
	li.mu.Lock()
	defer li.mu.Unlock()

	jobKey := jobID.String()
	if _, ok := li.watchers[jobKey]; ok {
		return
	}

	ctx, cancel := context.WithCancel(context.Background())
	li.watchers[jobKey] = cancel

	go li.runLogMonitoringLoop(ctx, jobID, since)
}

func (li *UserFacingLogInteractor) runLogMonitoringLoop(ctx context.Context, jobID id.JobID, since time.Time) {
	if err := li.checkPermission(ctx, rbac.ActionAny); err != nil {
		return
	}

	ticker := time.NewTicker(15 * time.Second)
	defer ticker.Stop()

	jobKey := jobID.String()
	latest := since.UTC()

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			if li.subscriptions.CountSubscribers(jobKey) == 0 {
				li.stopWatchingLogs(jobKey)
				return
			}

			currentJob, err := li.jobRepo.FindByID(context.Background(), jobID)
			if err != nil {
				reearth_log.Warnfc(ctx, "userfacinglog: failed to get job status: %v", err)
				continue
			}

			if currentJob != nil {
				status := currentJob.Status()
				if status == job.StatusCompleted || status == job.StatusFailed ||
					status == job.StatusCancelled {
					reearth_log.Debugfc(ctx, "userfacinglog: job %s is in terminal state %s, stopping log monitoring",
						jobID, status)
					li.stopWatchingLogs(jobKey)
					return
				}
			}

			now := time.Now().UTC()
			newLogs, err := li.logsGatewayRedis.GetUserFacingLogs(ctx, latest, now, jobID)
			if err != nil {
				reearth_log.Warnfc(ctx, "userfacinglog: failed to get logs in subscription: %v", err)
				continue
			}
			if len(newLogs) > 0 {
				li.subscriptions.Notify(jobKey, newLogs)
			}
			latest = now
		}
	}
}

func (li *UserFacingLogInteractor) stopWatchingLogs(jobKey string) {
	li.mu.Lock()
	defer li.mu.Unlock()

	if cancel, ok := li.watchers[jobKey]; ok {
		cancel()
		delete(li.watchers, jobKey)
	}
}

func (li *UserFacingLogInteractor) Unsubscribe(jobID id.JobID, ch chan *userfacinglog.UserFacingLog) {
	li.subscriptions.Unsubscribe(jobID.String(), ch)
}
