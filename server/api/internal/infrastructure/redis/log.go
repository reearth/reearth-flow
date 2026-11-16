package redis

import (
	"context"
	"crypto/sha256"
	"encoding/json"
	"errors"
	"fmt"
	"strconv"
	"strings"
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

// streamClockSkew widens the XRANGE lower bound to tolerate engine/Redis clock skew.
const streamClockSkew = time.Minute

// streamPageSize bounds each XRANGE reply.
const streamPageSize = 1000

// readJobStream pages through a per-job stream from since and calls fn once
// per distinct payload (Pub/Sub redelivery can append duplicates).
func (r *redisLog) readJobStream(ctx context.Context, streamKey string, since time.Time, fn func(data string)) error {
	start := "-"
	if ms := since.Add(-streamClockSkew).UnixMilli(); ms > 0 {
		start = strconv.FormatInt(ms, 10)
	}

	// Keyed by digest so the map does not keep every payload alive.
	seen := make(map[[sha256.Size]byte]struct{})
	for {
		// XRANGE on a missing key returns an empty slice, not redis.Nil.
		msgs, err := r.client.XRangeN(ctx, streamKey, start, "+", streamPageSize).Result()
		if err != nil {
			return fmt.Errorf("failed to xrange redis stream=%s: %w", streamKey, err)
		}

		for _, msg := range msgs {
			data, ok := msg.Values["data"].(string)
			if !ok {
				reearth_log.Warnfc(ctx, "redis: stream entry %s in %s has no data field", msg.ID, streamKey)
				continue
			}
			digest := sha256.Sum256([]byte(data))
			if _, dup := seen[digest]; dup {
				continue
			}
			seen[digest] = struct{}{}
			fn(data)
		}

		if len(msgs) < streamPageSize {
			return nil
		}
		next, err := nextStreamID(msgs[len(msgs)-1].ID)
		if err != nil {
			return fmt.Errorf("invalid stream id in %s: %w", streamKey, err)
		}
		start = next
	}
}

// nextStreamID returns the smallest stream ID after id ("ms-seq").
func nextStreamID(id string) (string, error) {
	ms, seq, ok := strings.Cut(id, "-")
	if !ok {
		return "", fmt.Errorf("malformed id %q", id)
	}
	n, err := strconv.ParseUint(seq, 10, 64)
	if err != nil {
		return "", fmt.Errorf("malformed id %q: %w", id, err)
	}
	return ms + "-" + strconv.FormatUint(n+1, 10), nil
}

func (r *redisLog) GetLogs(
	ctx context.Context,
	since time.Time,
	until time.Time,
	jobID id.JobID,
) ([]*log.Log, error) {
	sinceUTC := since.UTC()
	untilUTC := until.UTC()

	var result []*log.Log
	err := r.readJobStream(ctx, fmt.Sprintf("log:%s", jobID.String()), since, func(val string) {
		var entry LogEntry
		if err := json.Unmarshal([]byte(val), &entry); err != nil {
			reearth_log.Warnfc(ctx, "gql: failed to unmarshal log entry: %s", val)
			return
		}

		entryTimestampUTC := entry.Timestamp.UTC()
		if entryTimestampUTC.Before(sinceUTC) || entryTimestampUTC.After(untilUTC) {
			return
		}

		domainLog, err := entry.ToDomain()
		if err != nil {
			reearth_log.Warnfc(ctx, "gql: failed to convert log entry to domain: %v", err)
			return
		}
		result = append(result, domainLog)
	})
	if err != nil {
		return nil, err
	}

	return result, nil
}
