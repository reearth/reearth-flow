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

func NewRedisLog(client *redis.Client) (gateway.Redis, error) {
	if client == nil {
		return nil, errors.New("client is nil")
	}
	return &redisLog{client: client}, nil
}

type LogEntry struct {
	WorkflowID string    `json:"workflowId"`
	JobID      string    `json:"jobId"`
	NodeID     *string   `json:"nodeId,omitempty"`
	Timestamp  time.Time `json:"timestamp"`
	LogLevel   log.Level `json:"logLevel"`
	Message    string    `json:"message"`
}

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
		JobID:     l.JobID().String(),
		NodeID:    nid,
		Timestamp: l.Timestamp().UTC(),
		LogLevel:  l.Level(),
		Message:   l.Message(),
	}
}

func (e *LogEntry) ToDomain() (*log.Log, error) {
	jid, err := id.JobIDFrom(e.JobID)
	if err != nil {
		return nil, err
	}
	var nodeID *log.NodeID
	if e.NodeID != nil && *e.NodeID != "" {
		nid, err := id.NodeIDFrom(*e.NodeID)
		if err != nil {
			reearth_log.Warnf("gql: invalid node ID in log entry: %v", *e.NodeID)
		} else {
			nodeID = &nid
		}
	}

	return log.NewLog(
		jid,
		nodeID,
		e.Timestamp.UTC(),
		e.LogLevel,
		e.Message,
	), nil
}

func (r *redisLog) GetLogs(
	ctx context.Context,
	since time.Time,
	until time.Time,
	jobID id.JobID,
) ([]*log.Log, error) {
	// pattern: log:{workflowID}:{jobID}:*
	pattern := fmt.Sprintf("log:*:%s:*", jobID.String())

	var cursor uint64
	var result []*log.Log

	sinceUTC := since.UTC()
	untilUTC := until.UTC()

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

			var entry LogEntry
			if err := json.Unmarshal([]byte(val), &entry); err != nil {
				reearth_log.Warnfc(ctx, "gql: failed to unmarshal log entry: %s", val)
				continue
			}

			entryTimestampUTC := entry.Timestamp.UTC()

			if entryTimestampUTC.Before(sinceUTC) {
				continue
			}
			if entryTimestampUTC.After(untilUTC) {
				continue
			}

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
