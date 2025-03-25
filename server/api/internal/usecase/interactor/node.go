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
	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/subscription"
	"github.com/reearth/reearthx/log"
)

type NodeExecution struct {
	nodeRepo          repo.NodeExecution
	redisGateway      gateway.Redis
	subscriptions     *subscription.NodeManager
	watchers          map[string]context.CancelFunc
	mu                sync.Mutex
	permissionChecker gateway.PermissionChecker
}

func NewNodeExecution(nodeRepo repo.NodeExecution, redisGateway gateway.Redis, permissionChecker gateway.PermissionChecker) interfaces.NodeExecution {
	ee := &NodeExecution{
		nodeRepo:          nodeRepo,
		redisGateway:      redisGateway,
		subscriptions:     subscription.NewNodeManager(),
		watchers:          make(map[string]context.CancelFunc),
		permissionChecker: permissionChecker,
	}
	return ee
}

func (i *NodeExecution) checkPermission(ctx context.Context, action string) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceJob, action)
}

func (i *NodeExecution) FindByJobNodeID(ctx context.Context, id id.JobID, nodeID string) (*graph.NodeExecution, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	node, err := i.nodeRepo.FindByJobNodeID(ctx, id, nodeID)
	if err != nil {
		return nil, err
	}

	return node, nil
}

func (ei *NodeExecution) GetNodeExecutions(ctx context.Context, jobID id.JobID) ([]*graph.NodeExecution, error) {
	if err := ei.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	ctx, cancel := context.WithTimeout(ctx, 10*time.Second)
	defer cancel()

	if ei.redisGateway == nil {
		return nil, fmt.Errorf("redisGateway is nil: unable to get node executions from Redis")
	}

	nodes, err := ei.redisGateway.GetNodeExecutions(ctx, jobID)
	if err != nil {
		return nil, fmt.Errorf("failed to get node executions from Redis: %w", err)
	}

	return nodes, nil
}

func (ei *NodeExecution) GetNodeExecution(ctx context.Context, jobID id.JobID, nodeID string) (*graph.NodeExecution, error) {
	if err := ei.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	if ei.redisGateway == nil {
		return nil, fmt.Errorf("redisGateway is nil")
	}

	nodeExec, err := ei.redisGateway.GetNodeExecution(ctx, jobID, nodeID)
	if err != nil {
		return nil, fmt.Errorf("failed to get node execution from Redis: %w", err)
	}

	return nodeExec, nil
}

func (ei *NodeExecution) SubscribeToNode(ctx context.Context, jobID id.JobID, nodeID string) (chan *graph.NodeExecution, error) {
	if err := ei.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	if ei.redisGateway == nil {
		return nil, fmt.Errorf("redisGateway is nil")
	}

	key := fmt.Sprintf("%s:%s", jobID.String(), nodeID)
	ch := ei.subscriptions.Subscribe(key)

	go func() {
		nodeExec, err := ei.redisGateway.GetNodeExecution(context.Background(), jobID, nodeID)
		if err == nil && nodeExec != nil {
			ei.subscriptions.Notify(key, []*graph.NodeExecution{nodeExec})
		}
	}()

	ei.startWatchingNodeIfNeeded(jobID, nodeID)

	return ch, nil
}

func (ei *NodeExecution) startWatchingNodeIfNeeded(jobID id.JobID, nodeID string) {
	key := fmt.Sprintf("%s:%s", jobID.String(), nodeID)

	ei.mu.Lock()
	defer ei.mu.Unlock()

	if _, ok := ei.watchers[key]; ok {
		return
	}

	ctx, cancel := context.WithCancel(context.Background())
	ei.watchers[key] = cancel

	go ei.runNodeMonitoringLoop(ctx, jobID, nodeID)
}

func (ei *NodeExecution) runNodeMonitoringLoop(ctx context.Context, jobID id.JobID, nodeID string) {
	if err := ei.checkPermission(ctx, rbac.ActionAny); err != nil {
		return
	}

	key := fmt.Sprintf("%s:%s", jobID.String(), nodeID)
	ticker := time.NewTicker(3 * time.Second)
	defer ticker.Stop()

	var lastStatus graph.Status
	var initialFetch bool

	nodeExec, err := ei.redisGateway.GetNodeExecution(ctx, jobID, nodeID)
	if err == nil && nodeExec != nil {
		lastStatus = nodeExec.Status()
		initialFetch = true
	}

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			if ei.subscriptions.CountSubscribers(key) == 0 {
				ei.stopWatchingNode(key)
				return
			}

			currentNodeExec, err := ei.redisGateway.GetNodeExecution(ctx, jobID, nodeID)
			if err != nil {
				log.Warnfc(ctx, "node: failed to get node execution: %v", err)
				continue
			}

			if currentNodeExec == nil {
				continue
			}

			currentStatus := currentNodeExec.Status()

			if currentStatus != lastStatus || !initialFetch {
				if initialFetch {
					log.Debugfc(ctx, "node: status changed from %s to %s for job %s, node %s",
						lastStatus, currentStatus, jobID, nodeID)
				} else {
					initialFetch = true
				}

				lastStatus = currentStatus
				ei.subscriptions.Notify(key, []*graph.NodeExecution{currentNodeExec})
			}

			if currentStatus == graph.StatusCompleted ||
				currentStatus == graph.StatusFailed {
				log.Debugfc(ctx, "node: monitoring stopped for job %s, node %s (status: %s)",
					jobID, nodeID, currentStatus)
				ei.stopWatchingNode(key)
				return
			}
		}
	}
}

func (ei *NodeExecution) stopWatchingNode(key string) {
	ei.mu.Lock()
	defer ei.mu.Unlock()

	if cancel, ok := ei.watchers[key]; ok {
		cancel()
		delete(ei.watchers, key)
	}
}

func (ei *NodeExecution) UnsubscribeFromNode(jobID id.JobID, nodeID string, ch chan *graph.NodeExecution) {
	key := fmt.Sprintf("%s:%s", jobID.String(), nodeID)
	ei.subscriptions.Unsubscribe(key, ch)
}
