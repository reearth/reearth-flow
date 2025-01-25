package redis

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"time"

	"github.com/redis/go-redis/v9"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
	reearth_log "github.com/reearth/reearthx/log"
)

type redisLog struct {
	client *redis.Client
}

func NewRedisLog(client *redis.Client) (gateway.Log, error) {
	if client == nil {
		return nil, errors.New("client is nil")
	}

	return &redisLog{client: client}, nil
}

type LogEntry struct {
	WorkflowID string    `json:"workflowId"`
	JobID      string    `json:"jobId"`
	NodeID     *string   `json:"nodeId,omitempty"`
	LoggedAt   time.Time `json:"timestamp"`
	LogLevel   log.Level `json:"logLevel"`
	Message    string    `json:"message"`
}

// Domain log.Log -> LogEntry conversion
func ToLogEntry(l *log.Log) *LogEntry {
	if l == nil {
		return nil
	}
	var nid *string
	if l.NodeID() != nil {
		s := l.NodeID().String()
		nid = &s
	}
	return &LogEntry{
		WorkflowID: l.WorkflowID().String(),
		JobID:      l.JobID().String(),
		NodeID:     nid,
		LoggedAt:   l.Timestamp(),
		LogLevel:   l.Level(),
		Message:    l.Message(),
	}
}

// LogEntry -> domain log.Log conversion
func (e *LogEntry) ToDomain() (*log.Log, error) {
	wid, err := id.WorkflowIDFrom(e.WorkflowID)
	if err != nil {
		return nil, err
	}
	jid, err := id.JobIDFrom(e.JobID)
	if err != nil {
		return nil, err
	}
	var nodeID *log.NodeID
	if e.NodeID != nil && *e.NodeID != "" {
		nid, err := id.NodeIDFrom(*e.NodeID)
		if err == nil {
			nodeID = &nid
		}
	}

	return log.NewLog(
		wid,
		jid,
		nodeID,
		e.LoggedAt,
		e.LogLevel,
		e.Message,
	), nil
}

func (r *redisLog) GetLogs(ctx context.Context, since time.Time, workflowID id.WorkflowID, jobID id.JobID) ([]*log.Log, error) {
	// pattern: log:{workflowID}:{jobID}:*
	pattern := fmt.Sprintf("log:%s:%s:*", workflowID.String(), jobID.String())

	var cursor uint64
	var result []*log.Log

	// Scan for keys using SCAN command
	for {
		// count=100 is the maximum number of items returned by one SCAN. Adjust as needed.
		keys, newCursor, err := r.client.Scan(ctx, cursor, pattern, 100).Result()
		if err != nil {
			return nil, fmt.Errorf("failed to scan redis keys: %w", err)
		}

		for _, key := range keys {
			// Get value (JSON string) from key
			val, err := r.client.Get(ctx, key).Result()
			if err == redis.Nil {
				// Skip if key does not exist (deleted)
				continue
			} else if err != nil {
				return nil, fmt.Errorf("failed to get redis value for key=%s: %w", key, err)
			}

			// JSON -> LogEntry
			var entry LogEntry
			if err := json.Unmarshal([]byte(val), &entry); err != nil {
				// Skip if corrupted JSON
				reearth_log.Warnfc(ctx, "gql: failed to unmarshal log entry: %s", val)
				continue
			}

			// Exclude anything older than "since" timestamp
			if entry.LoggedAt.Before(since) {
				continue
			}

			// Convert to domain log.Log
			domainLog, err := entry.ToDomain()
			if err != nil {
				reearth_log.Warnfc(ctx, "gql: failed to convert log entry to domain: %v", err)
				continue
			}
			result = append(result, domainLog)
		}

		cursor = newCursor
		if cursor == 0 {
			break
		}
	}

	return result, nil
}
