package redis

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/redis/go-redis/v9"
	"github.com/reearth/reearth-flow/api/pkg/edge"
	"github.com/reearth/reearth-flow/api/pkg/id"
	reearth_log "github.com/reearth/reearthx/log"
)

type EdgeEntry struct {
	ID                  string     `json:"id"`
	WorkflowID          string     `json:"workflowId"`
	JobID               string     `json:"jobId"`
	Status              string     `json:"status"`
	Timestamp           time.Time  `json:"timestamp"`
	StartedAt           *time.Time `json:"startedAt,omitempty"`
	CompletedAt         *time.Time `json:"completedAt,omitempty"`
	FeatureID           *string    `json:"featureId,omitempty"`
	IntermediateDataURL string     `json:"intermediateDataUrl,omitempty"`
}

func (e *EdgeEntry) ToDomain() (*edge.EdgeExecution, error) {
	jid, err := id.JobIDFrom(e.JobID)
	if err != nil {
		return nil, err
	}

	return edge.NewEdgeExecution(
		e.ID,
		jid,
		e.WorkflowID,
		edge.Status(e.Status),
		e.StartedAt,
		e.CompletedAt,
		e.FeatureID,
		&e.IntermediateDataURL,
	), nil
}

func (r *redisLog) GetEdgeExecutions(
	ctx context.Context,
	jobID id.JobID,
) ([]*edge.EdgeExecution, error) {
	// pattern: edge:{jobID}:*
	pattern := fmt.Sprintf("edge:%s:*", jobID.String())
	var cursor uint64
	var result []*edge.EdgeExecution

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

			var entry EdgeEntry
			if err := json.Unmarshal([]byte(val), &entry); err != nil {
				reearth_log.Warnfc(ctx, "redis: failed to unmarshal edge entry: %s", val)
				continue
			}

			domainEdge, err := entry.ToDomain()
			if err != nil {
				reearth_log.Warnfc(ctx, "redis: failed to convert edge entry to domain: %v", err)
				continue
			}

			result = append(result, domainEdge)
		}

		cursor = newCursor
		if cursor == 0 {
			break
		}
	}

	return result, nil
}

func (r *redisLog) GetEdgeExecution(
	ctx context.Context,
	jobID id.JobID,
	edgeID string,
) (*edge.EdgeExecution, error) {
	// key: edge:{jobID}:{edgeID}
	key := fmt.Sprintf("edge:%s:%s", jobID.String(), edgeID)

	val, err := r.client.Get(ctx, key).Result()
	if err == redis.Nil {
		return nil, fmt.Errorf("edge execution not found: %s", key)
	} else if err != nil {
		return nil, fmt.Errorf("failed to get redis value for key=%s: %w", key, err)
	}

	var entry EdgeEntry
	if err := json.Unmarshal([]byte(val), &entry); err != nil {
		return nil, fmt.Errorf("failed to unmarshal edge entry: %w", err)
	}

	domainEdge, err := entry.ToDomain()
	if err != nil {
		return nil, fmt.Errorf("failed to convert edge entry to domain: %w", err)
	}

	return domainEdge, nil
}
