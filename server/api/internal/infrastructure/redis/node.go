package redis

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/redis/go-redis/v9"
	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
	reearth_log "github.com/reearth/reearthx/log"
)

type NodeEntry struct {
	ID          string     `json:"id"`
	WorkflowID  string     `json:"workflowId"`
	JobID       string     `json:"jobId"`
	NodeID      string     `json:"nodeId"`
	Status      string     `json:"status"`
	StartedAt   *time.Time `json:"startedAt,omitempty"`
	CompletedAt *time.Time `json:"completedAt,omitempty"`
	FeatureID   *string    `json:"featureId,omitempty"`
	Timestamp   time.Time  `json:"timestamp"`
}

func (e *NodeEntry) ToDomain() (*graph.NodeExecution, error) {
	jId, err := id.JobIDFrom(e.JobID)
	if err != nil {
		return nil, err
	}

	nEId, err := id.NodeExecutionIDFrom(e.ID)
	if err != nil {
		return nil, err
	}

	nId, err := id.NodeIDFrom(e.NodeID)
	if err != nil {
		return nil, err
	}

	return graph.NewNodeExecution(
		nEId,
		jId,
		nId,
		graph.Status(e.Status),
	), nil
}

func (r *redisLog) GetNodeExecutions(
	ctx context.Context,
	jobID id.JobID,
) ([]*graph.NodeExecution, error) {
	// pattern: node:{jobID}:*
	pattern := fmt.Sprintf("node:%s:*", jobID.String())
	var cursor uint64
	var result []*graph.NodeExecution

	for {
		keys, newCursor, err := r.client.Scan(ctx, cursor, pattern, 100).Result()
		if err != nil {
			return nil, fmt.Errorf("failed to scan redis keys: %w", err)
		}

		for _, key := range keys {
			val, err := r.client.Get(ctx, key).Result()
			if err == redis.Nil {
				continue
			} else if err != nil {
				return nil, fmt.Errorf("failed to get redis value for key=%s: %w", key, err)
			}

			var entry NodeEntry
			if err := json.Unmarshal([]byte(val), &entry); err != nil {
				reearth_log.Warnfc(ctx, "redis: failed to unmarshal node entry: %s", val)
				continue
			}

			domainNode, err := entry.ToDomain()
			if err != nil {
				reearth_log.Warnfc(ctx, "redis: failed to convert node entry to domain: %v", err)
				continue
			}

			result = append(result, domainNode)
		}

		cursor = newCursor
		if cursor == 0 {
			break
		}
	}

	return result, nil
}

func (r *redisLog) GetNodeExecution(
	ctx context.Context,
	jobID id.JobID,
	nodeID string,
) (*graph.NodeExecution, error) {
	// key: node:{jobID}:{nodeID}
	key := fmt.Sprintf("node:%s:%s", jobID.String(), nodeID)

	val, err := r.client.Get(ctx, key).Result()
	if err == redis.Nil {
		return nil, fmt.Errorf("node execution not found: %s", key)
	} else if err != nil {
		return nil, fmt.Errorf("failed to get redis value for key=%s: %w", key, err)
	}

	var entry NodeEntry
	if err := json.Unmarshal([]byte(val), &entry); err != nil {
		return nil, fmt.Errorf("failed to unmarshal node entry: %w", err)
	}

	domainNode, err := entry.ToDomain()
	if err != nil {
		return nil, fmt.Errorf("failed to convert node entry to domain: %w", err)
	}

	return domainNode, nil
}
