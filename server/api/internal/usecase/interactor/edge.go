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
	"github.com/reearth/reearth-flow/api/pkg/edge"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/subscription"
	"github.com/reearth/reearthx/log"
)

type EdgeExecution struct {
	edgeRepo          repo.EdgeExecution
	redisGateway      gateway.Redis
	subscriptions     *subscription.EdgeManager
	watchers          map[string]context.CancelFunc
	mu                sync.Mutex
	permissionChecker gateway.PermissionChecker
}

func NewEdgeExecution(redisGateway gateway.Redis, permissionChecker gateway.PermissionChecker) interfaces.EdgeExecution {
	ee := &EdgeExecution{
		redisGateway:      redisGateway,
		subscriptions:     subscription.NewEdgeManager(),
		watchers:          make(map[string]context.CancelFunc),
		permissionChecker: permissionChecker,
	}
	return ee
}

func (i *EdgeExecution) checkPermission(ctx context.Context, action string) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceJob, action)
}

func (i *EdgeExecution) FindByJobEdgeID(ctx context.Context, id id.JobID, edgeID string) (*edge.EdgeExecution, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	edge, err := i.edgeRepo.FindByJobEdgeID(ctx, id, edgeID)
	if err != nil {
		return nil, err
	}

	return edge, nil
}

func (ei *EdgeExecution) GetEdgeExecutions(ctx context.Context, jobID id.JobID) ([]*edge.EdgeExecution, error) {
	if err := ei.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	ctx, cancel := context.WithTimeout(ctx, 10*time.Second)
	defer cancel()

	if ei.redisGateway == nil {
		return nil, fmt.Errorf("redisGateway is nil: unable to get edge executions from Redis")
	}

	edges, err := ei.redisGateway.GetEdgeExecutions(ctx, jobID)
	if err != nil {
		return nil, fmt.Errorf("failed to get edge executions from Redis: %w", err)
	}

	return edges, nil
}

func (ei *EdgeExecution) GetEdgeExecution(ctx context.Context, jobID id.JobID, edgeID string) (*edge.EdgeExecution, error) {
	if err := ei.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	if ei.redisGateway == nil {
		return nil, fmt.Errorf("redisGateway is nil")
	}

	edgeExec, err := ei.redisGateway.GetEdgeExecution(ctx, jobID, edgeID)
	if err != nil {
		return nil, fmt.Errorf("failed to get edge execution from Redis: %w", err)
	}

	return edgeExec, nil
}

func (ei *EdgeExecution) SubscribeToEdge(ctx context.Context, jobID id.JobID, edgeID string) (chan *edge.EdgeExecution, error) {
	if err := ei.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	if ei.redisGateway == nil {
		return nil, fmt.Errorf("redisGateway is nil")
	}

	key := fmt.Sprintf("%s:%s", jobID.String(), edgeID)
	ch := ei.subscriptions.Subscribe(key)

	go func() {
		edgeExec, err := ei.redisGateway.GetEdgeExecution(context.Background(), jobID, edgeID)
		if err == nil && edgeExec != nil {
			ei.subscriptions.Notify(key, []*edge.EdgeExecution{edgeExec})
		}
	}()

	ei.startWatchingEdgeIfNeeded(jobID, edgeID)

	return ch, nil
}

func (ei *EdgeExecution) startWatchingEdgeIfNeeded(jobID id.JobID, edgeID string) {
	key := fmt.Sprintf("%s:%s", jobID.String(), edgeID)

	ei.mu.Lock()
	defer ei.mu.Unlock()

	if _, ok := ei.watchers[key]; ok {
		return
	}

	ctx, cancel := context.WithCancel(context.Background())
	ei.watchers[key] = cancel

	go ei.runEdgeMonitoringLoop(ctx, jobID, edgeID)
}

func (ei *EdgeExecution) runEdgeMonitoringLoop(ctx context.Context, jobID id.JobID, edgeID string) {
	if err := ei.checkPermission(ctx, rbac.ActionAny); err != nil {
		return
	}

	key := fmt.Sprintf("%s:%s", jobID.String(), edgeID)
	ticker := time.NewTicker(3 * time.Second)
	defer ticker.Stop()

	var lastStatus edge.Status
	var initialFetch bool

	edgeExec, err := ei.redisGateway.GetEdgeExecution(ctx, jobID, edgeID)
	if err == nil && edgeExec != nil {
		lastStatus = edgeExec.Status()
		initialFetch = true
	}

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			if ei.subscriptions.CountSubscribers(key) == 0 {
				ei.stopWatchingEdge(key)
				return
			}

			currentEdgeExec, err := ei.redisGateway.GetEdgeExecution(ctx, jobID, edgeID)
			if err != nil {
				log.Warnfc(ctx, "edge: failed to get edge execution: %v", err)
				continue
			}

			if currentEdgeExec == nil {
				continue
			}

			currentStatus := currentEdgeExec.Status()

			if currentStatus != lastStatus || !initialFetch {
				if initialFetch {
					log.Debugfc(ctx, "edge: status changed from %s to %s for job %s, edge %s",
						lastStatus, currentStatus, jobID, edgeID)
				} else {
					initialFetch = true
				}

				lastStatus = currentStatus
				ei.subscriptions.Notify(key, []*edge.EdgeExecution{currentEdgeExec})
			}

			if currentStatus == edge.StatusCompleted ||
				currentStatus == edge.StatusFailed {
				log.Debugfc(ctx, "edge: monitoring stopped for job %s, edge %s (status: %s)",
					jobID, edgeID, currentStatus)
				ei.stopWatchingEdge(key)
				return
			}
		}
	}
}

func (ei *EdgeExecution) stopWatchingEdge(key string) {
	ei.mu.Lock()
	defer ei.mu.Unlock()

	if cancel, ok := ei.watchers[key]; ok {
		cancel()
		delete(ei.watchers, key)
	}
}

func (ei *EdgeExecution) UnsubscribeFromEdge(jobID id.JobID, edgeID string, ch chan *edge.EdgeExecution) {
	key := fmt.Sprintf("%s:%s", jobID.String(), edgeID)
	ei.subscriptions.Unsubscribe(key, ch)
}
