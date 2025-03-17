package interactor

import (
	"context"
	"fmt"
	"log"
	"sync"
	"time"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/edge"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/subscription"
	reearth_log "github.com/reearth/reearthx/log"
)

type EdgeExecution struct {
	jobRepo           repo.Job
	redisGateway      gateway.Redis
	subscriptions     *subscription.EdgeManager
	watchers          map[string]context.CancelFunc
	mu                sync.Mutex
	permissionChecker gateway.PermissionChecker
}

func NewEdgeExecution(redisGateway gateway.Redis, permissionChecker gateway.PermissionChecker) interfaces.EdgeExecution {
	log.Printf("DEBUG: Creating new EdgeExecution interactor")
	ee := &EdgeExecution{
		redisGateway:      redisGateway,
		subscriptions:     subscription.NewEdgeManager(),
		watchers:          make(map[string]context.CancelFunc),
		permissionChecker: permissionChecker,
	}
	log.Printf("DEBUG: EdgeExecution interactor created with redisGateway=%v", redisGateway != nil)
	return ee
}

func (ei *EdgeExecution) checkPermission(ctx context.Context, action string) error {
	log.Printf("DEBUG: Checking permission for action %s on resource %s", action, rbac.ResourceEdge)
	err := checkPermission(ctx, ei.permissionChecker, rbac.ResourceEdge, action)
	if err != nil {
		log.Printf("WARN: Permission check failed: %v", err)
	} else {
		log.Printf("DEBUG: Permission check passed for action %s", action)
	}
	return err
}

func (i *EdgeExecution) FindByEdgeID(ctx context.Context, id id.JobID, edgeID string) (*edge.EdgeExecution, error) {
	log.Printf("DEBUG: FindByEdgeID called for jobID=%s, edgeID=%s", id.String(), edgeID)
	
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		log.Printf("ERROR: Permission denied for FindByEdgeID: %v", err)
		return nil, err
	}

	log.Printf("DEBUG: Looking up job with ID %s", id.String())
	j, err := i.jobRepo.FindByID(ctx, id)
	if err != nil {
		log.Printf("ERROR: Failed to find job %s: %v", id.String(), err)
		return nil, err
	}

	if j == nil {
		log.Printf("ERROR: Job %s not found", id.String())
		return nil, fmt.Errorf("job not found")
	}

	log.Printf("DEBUG: Found job with ID %s, scanning %d edge executions", id.String(), len(j.EdgeExecutions()))
	for idx, e := range j.EdgeExecutions() {
		log.Printf("DEBUG: Checking edge %d/%d with ID %s", idx+1, len(j.EdgeExecutions()), e.ID())
		if e.ID() == edgeID {
			log.Printf("DEBUG: Found matching edge with ID %s and status %s", e.ID(), e.Status())
			return e, nil
		}
	}

	log.Printf("ERROR: Edge %s not found in job %s", edgeID, id.String())
	return nil, fmt.Errorf("edge not found")
}

func (ei *EdgeExecution) GetEdgeExecutions(ctx context.Context, jobID id.JobID) ([]*edge.EdgeExecution, error) {
	log.Printf("DEBUG: GetEdgeExecutions called for jobID=%s", jobID.String())
	
	if err := ei.checkPermission(ctx, rbac.ActionAny); err != nil {
		log.Printf("ERROR: Permission denied for GetEdgeExecutions: %v", err)
		return nil, err
	}

	log.Printf("DEBUG: Creating timeout context (10s) for Redis operation")
	ctx, cancel := context.WithTimeout(ctx, 10*time.Second)
	defer cancel()

	if ei.redisGateway == nil {
		log.Printf("ERROR: redisGateway is nil in GetEdgeExecutions")
		reearth_log.Error("redisGateway is nil: unable to get edge executions from Redis")
		return nil, fmt.Errorf("redisGateway is nil: unable to get edge executions from Redis")
	}

	log.Printf("DEBUG: Fetching edge executions from Redis for jobID=%s", jobID.String())
	edges, err := ei.redisGateway.GetEdgeExecutions(ctx, jobID)
	if err != nil {
		log.Printf("ERROR: Failed to get edge executions from Redis for jobID=%s: %v", jobID.String(), err)
		return nil, fmt.Errorf("failed to get edge executions from Redis: %w", err)
	}

	log.Printf("DEBUG: Successfully retrieved %d edge executions from Redis for jobID=%s", len(edges), jobID.String())
	for i, edge := range edges {
		log.Printf("DEBUG: Edge %d/%d: ID=%s, Status=%s", i+1, len(edges), edge.ID(), edge.Status())
	}
	
	return edges, nil
}

func (ei *EdgeExecution) GetEdgeExecution(ctx context.Context, jobID id.JobID, edgeID string) (*edge.EdgeExecution, error) {
	log.Printf("DEBUG: GetEdgeExecution called for jobID=%s, edgeID=%s", jobID.String(), edgeID)
	
	if err := ei.checkPermission(ctx, rbac.ActionAny); err != nil {
		log.Printf("ERROR: Permission denied for GetEdgeExecution: %v", err)
		return nil, err
	}

	log.Printf("DEBUG: Creating timeout context (5s) for Redis operation")
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	if ei.redisGateway == nil {
		log.Printf("ERROR: redisGateway is nil in GetEdgeExecution")
		return nil, fmt.Errorf("redisGateway is nil")
	}

	log.Printf("DEBUG: Fetching edge execution from Redis for jobID=%s, edgeID=%s", jobID.String(), edgeID)
	edgeExec, err := ei.redisGateway.GetEdgeExecution(ctx, jobID, edgeID)
	if err != nil {
		log.Printf("ERROR: Failed to get edge execution from Redis for jobID=%s, edgeID=%s: %v", 
			jobID.String(), edgeID, err)
		return nil, fmt.Errorf("failed to get edge execution from Redis: %w", err)
	}

	if edgeExec != nil {
		log.Printf("DEBUG: Successfully retrieved edge execution: ID=%s, Status=%s", edgeExec.ID(), edgeExec.Status())
	} else {
		log.Printf("DEBUG: Edge execution not found in Redis")
	}
	
	return edgeExec, nil
}

func (ei *EdgeExecution) SubscribeToEdge(ctx context.Context, jobID id.JobID, edgeID string) (chan *edge.EdgeExecution, error) {
	log.Printf("DEBUG: SubscribeToEdge called for jobID=%s, edgeID=%s", jobID.String(), edgeID)
	
	if err := ei.checkPermission(ctx, rbac.ActionAny); err != nil {
		log.Printf("ERROR: Permission denied for SubscribeToEdge: %v", err)
		return nil, err
	}

	if ei.redisGateway == nil {
		log.Printf("ERROR: redisGateway is nil in SubscribeToEdge")
		return nil, fmt.Errorf("redisGateway is nil")
	}

	key := fmt.Sprintf("%s:%s", jobID.String(), edgeID)
	log.Printf("DEBUG: Creating subscription key: %s", key)

	log.Printf("DEBUG: Creating new subscription channel")
	ch := ei.subscriptions.Subscribe(key)
	log.Printf("DEBUG: Subscription channel created, key=%s", key)

	go func() {
		log.Printf("DEBUG: Fetching initial edge state for notification, key=%s", key)
		edgeExec, err := ei.redisGateway.GetEdgeExecution(ctx, jobID, edgeID)
		if err != nil {
			log.Printf("WARN: Failed to get initial edge state for key=%s: %v", key, err)
			return
		}
		
		if edgeExec != nil {
			log.Printf("DEBUG: Sending initial edge notification, key=%s, status=%s", key, edgeExec.Status())
			ei.subscriptions.Notify(key, []*edge.EdgeExecution{edgeExec})
		} else {
			log.Printf("DEBUG: No initial edge state found for key=%s", key)
		}
	}()

	log.Printf("DEBUG: Starting edge monitoring if needed for key=%s", key)
	ei.startWatchingEdgeIfNeeded(jobID, edgeID)

	return ch, nil
}

func (ei *EdgeExecution) startWatchingEdgeIfNeeded(jobID id.JobID, edgeID string) {
	key := fmt.Sprintf("%s:%s", jobID.String(), edgeID)
	log.Printf("DEBUG: startWatchingEdgeIfNeeded called for key=%s", key)
	
	ei.mu.Lock()
	defer ei.mu.Unlock()

	if _, ok := ei.watchers[key]; ok {
		log.Printf("DEBUG: Edge watcher already exists for key=%s, skipping", key)
		return
	}

	log.Printf("DEBUG: Creating new edge watcher for key=%s", key)
	ctx, cancel := context.WithCancel(context.Background())
	ei.watchers[key] = cancel
	log.Printf("DEBUG: Edge watcher registered with cancel function, key=%s", key)

	log.Printf("DEBUG: Starting edge monitoring loop for key=%s", key)
	go ei.runEdgeMonitoringLoop(ctx, jobID, edgeID)
}

func (ei *EdgeExecution) runEdgeMonitoringLoop(ctx context.Context, jobID id.JobID, edgeID string) {
	key := fmt.Sprintf("%s:%s", jobID.String(), edgeID)
	log.Printf("DEBUG: Edge monitoring loop started for key=%s", key)
	
	if err := ei.checkPermission(ctx, rbac.ActionAny); err != nil {
		log.Printf("ERROR: Permission denied for edge monitoring loop, key=%s: %v", key, err)
		return
	}

	log.Printf("DEBUG: Creating ticker (3s interval) for key=%s", key)
	ticker := time.NewTicker(3 * time.Second)
	defer ticker.Stop()

	var lastStatus edge.Status

	// Get initial status
	log.Printf("DEBUG: Fetching initial edge status for key=%s", key)
	edgeExec, err := ei.redisGateway.GetEdgeExecution(ctx, jobID, edgeID)
	if err != nil {
		log.Printf("WARN: Failed to get initial edge status for key=%s: %v", key, err)
	} else if edgeExec != nil {
		lastStatus = edgeExec.Status()
		log.Printf("DEBUG: Initial edge status for key=%s: %s", key, lastStatus)
	} else {
		log.Printf("DEBUG: Initial edge execution not found for key=%s", key)
	}

	log.Printf("DEBUG: Entering monitoring loop for key=%s", key)
	loopCount := 0
	for {
		select {
		case <-ctx.Done():
			log.Printf("DEBUG: Context cancelled, stopping monitoring loop for key=%s", key)
			return
		case <-ticker.C:
			loopCount++
			
			// Check if we still have subscribers
			subscriberCount := ei.subscriptions.CountSubscribers(key)
			log.Printf("DEBUG: [Loop %d] Checking subscribers for key=%s: count=%d", 
				loopCount, key, subscriberCount)
			
			if subscriberCount == 0 {
				log.Printf("DEBUG: No more subscribers for key=%s, stopping monitoring", key)
				ei.stopWatchingEdge(key)
				return
			}

			// Poll for updates
			log.Printf("DEBUG: [Loop %d] Polling for updates for key=%s", loopCount, key)
			edgeExec, err := ei.redisGateway.GetEdgeExecution(ctx, jobID, edgeID)
			if err != nil {
				log.Printf("WARN: [Loop %d] Failed to get edge execution for key=%s: %v", loopCount, key, err)
				reearth_log.Warnfc(ctx, "edge: failed to get edge execution in subscription: %v", err)
				continue
			}

			if edgeExec == nil {
				log.Printf("DEBUG: [Loop %d] Edge execution not found for key=%s", loopCount, key)
				continue
			}

			currentStatus := edgeExec.Status()
			log.Printf("DEBUG: [Loop %d] Edge status for key=%s: current=%s, last=%s", 
				loopCount, key, currentStatus, lastStatus)
				
			if currentStatus != lastStatus {
				log.Printf("DEBUG: [Loop %d] Status changed for key=%s: %s -> %s, notifying subscribers", 
					loopCount, key, lastStatus, currentStatus)
				lastStatus = currentStatus
				ei.subscriptions.Notify(key, []*edge.EdgeExecution{edgeExec})
			}
		}
	}
}

func (ei *EdgeExecution) stopWatchingEdge(key string) {
	log.Printf("DEBUG: stopWatchingEdge called for key=%s", key)
	
	ei.mu.Lock()
	defer ei.mu.Unlock()

	if cancel, ok := ei.watchers[key]; ok {
		log.Printf("DEBUG: Cancelling context for edge watcher, key=%s", key)
		cancel()
		delete(ei.watchers, key)
		log.Printf("DEBUG: Edge watcher removed, key=%s", key)
	} else {
		log.Printf("DEBUG: No edge watcher found for key=%s", key)
	}
}

func (ei *EdgeExecution) UnsubscribeFromEdge(jobID id.JobID, edgeID string, ch chan *edge.EdgeExecution) {
	key := fmt.Sprintf("%s:%s", jobID.String(), edgeID)
	log.Printf("DEBUG: UnsubscribeFromEdge called for key=%s", key)
	
	beforeCount := ei.subscriptions.CountSubscribers(key)
	ei.subscriptions.Unsubscribe(key, ch)
	afterCount := ei.subscriptions.CountSubscribers(key)
	
	log.Printf("DEBUG: Unsubscribed from edge, key=%s, subscribers: %d -> %d", 
		key, beforeCount, afterCount)
		
	if afterCount == 0 {
		log.Printf("DEBUG: No more subscribers for key=%s, watcher will auto-terminate on next cycle", key)
	}
}
