package redis

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"strconv"
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

// streamClockSkew widens the XRANGE lower bound. Stream IDs are assigned by
// Redis at ingest time, which is never earlier than the event timestamp except
// for clock skew between the engine and Redis; entries are still filtered by
// their own timestamp afterwards.
const streamClockSkew = time.Minute

// readJobStream returns the serialized entries of a per-job stream written by
// the subscriber ("{prefix}:{jobID}"), starting from since minus clock skew.
func (r *redisLog) readJobStream(ctx context.Context, streamKey string, since time.Time) ([]string, error) {
	start := "-"
	if ms := since.Add(-streamClockSkew).UnixMilli(); ms > 0 {
		start = strconv.FormatInt(ms, 10)
	}

	// XRANGE on a missing key returns an empty slice, not redis.Nil.
	msgs, err := r.client.XRange(ctx, streamKey, start, "+").Result()
	if err != nil {
		return nil, fmt.Errorf("failed to xrange redis stream=%s: %w", streamKey, err)
	}

	vals := make([]string, 0, len(msgs))
	for _, msg := range msgs {
		data, ok := msg.Values["data"].(string)
		if !ok {
			reearth_log.Warnfc(ctx, "redis: stream entry %s in %s has no data field", msg.ID, streamKey)
			continue
		}
		vals = append(vals, data)
	}
	return vals, nil
}

func (r *redisLog) GetLogs(
	ctx context.Context,
	since time.Time,
	until time.Time,
	jobID id.JobID,
) ([]*log.Log, error) {
	vals, err := r.readJobStream(ctx, fmt.Sprintf("log:%s", jobID.String()), since)
	if err != nil {
		return nil, err
	}

	sinceUTC := since.UTC()
	untilUTC := until.UTC()

	var result []*log.Log
	for _, val := range vals {
		var entry LogEntry
		if err := json.Unmarshal([]byte(val), &entry); err != nil {
			reearth_log.Warnfc(ctx, "gql: failed to unmarshal log entry: %s", val)
			continue
		}

		entryTimestampUTC := entry.Timestamp.UTC()
		if entryTimestampUTC.Before(sinceUTC) || entryTimestampUTC.After(untilUTC) {
			continue
		}

		domainLog, err := entry.ToDomain()
		if err != nil {
			reearth_log.Warnfc(ctx, "gql: failed to convert log entry to domain: %v", err)
			continue
		}
		result = append(result, domainLog)
	}

	return result, nil
}
