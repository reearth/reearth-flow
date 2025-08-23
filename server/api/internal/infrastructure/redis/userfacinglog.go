package redis

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/redis/go-redis/v9"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/userfacinglog"
	reearth_log "github.com/reearth/reearthx/log"
)

type UserFacingLogEntry struct {
	WorkflowID string          `json:"workflowId"`
	JobID      string          `json:"jobId"`
	Timestamp  time.Time       `json:"timestamp"`
	Message    string          `json:"message"`
	Metadata   json.RawMessage `json:"metadata,omitempty"`
}

func ToUserFacingLogEntry(l *userfacinglog.UserFacingLog) *UserFacingLogEntry {
	if l == nil {
		return nil
	}

	return &UserFacingLogEntry{
		JobID:     l.JobID().String(),
		Timestamp: l.Timestamp().UTC(),
		Message:   l.Message(),
		Metadata:  l.Metadata(),
	}
}

func (e *UserFacingLogEntry) ToDomain() (*userfacinglog.UserFacingLog, error) {
	jid, err := id.JobIDFrom(e.JobID)
	if err != nil {
		return nil, err
	}

	return userfacinglog.NewUserFacingLog(
		jid,
		e.Timestamp.UTC(),
		e.Message,
		e.Metadata,
	), nil
}

func (r *redisLog) GetUserFacingLogs(
	ctx context.Context,
	since time.Time,
	until time.Time,
	jobID id.JobID,
) ([]*userfacinglog.UserFacingLog, error) {
	// pattern: userfacinglog:{workflowID}:{jobID}:*
	pattern := fmt.Sprintf("userfacinglog:*:%s:*", jobID.String())

	var cursor uint64
	var result []*userfacinglog.UserFacingLog

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

			var entry UserFacingLogEntry
			if err := json.Unmarshal([]byte(val), &entry); err != nil {
				reearth_log.Warnfc(ctx, "gql: failed to unmarshal user-facing log entry: %s", val)
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
				reearth_log.Warnfc(ctx, "gql: failed to convert user-facing log entry to domain: %v", err)
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
