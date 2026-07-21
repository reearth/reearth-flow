package redis

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
	reearth_log "github.com/reearth/reearthx/log"
)

// DiagnosticEntry is the wire shape pushed onto the diagnostics:{jobId}
// [:{nodeId}] Redis lists; WireDiagnostic is embedded, not nested, to match
// the subscriber's flattened JSON shape on Unmarshal.
type DiagnosticEntry struct {
	Timestamp  time.Time `json:"timestamp"`
	WorkflowID string    `json:"workflowId"`
	JobID      string    `json:"jobId"`
	Schema     string    `json:"schema"`
	gateway.WireDiagnostic
}

func (e *DiagnosticEntry) ToDomain() (*diagnostic.Diagnostic, error) {
	jid, err := id.JobIDFrom(e.JobID)
	if err != nil {
		return nil, err
	}
	return e.WireDiagnostic.ToDomain(jid, e.Timestamp.UTC())
}

// GetNodeDiagnostics reads diagnostics:{jobId}:{nodeId} (nodeID "" reads
// the "_job" bucket, mirroring the subscriber's fallback).
func (r *redisLog) GetNodeDiagnostics(
	ctx context.Context,
	jobID id.JobID,
	nodeID string,
) ([]*diagnostic.Diagnostic, error) {
	nodeSegment := nodeID
	if nodeSegment == "" {
		nodeSegment = "_job"
	}
	key := fmt.Sprintf("diagnostics:%s:%s", jobID.String(), nodeSegment)
	return r.getDiagnosticsList(ctx, key)
}

// GetJobDiagnostics reads diagnostics:{jobId}, the whole-job index list the
// subscriber double-writes every diagnostic event onto.
func (r *redisLog) GetJobDiagnostics(
	ctx context.Context,
	jobID id.JobID,
) ([]*diagnostic.Diagnostic, error) {
	key := fmt.Sprintf("diagnostics:%s", jobID.String())
	return r.getDiagnosticsList(ctx, key)
}

func (r *redisLog) getDiagnosticsList(ctx context.Context, key string) ([]*diagnostic.Diagnostic, error) {
	// LRANGE on a missing key returns an empty slice with no error (unlike
	// GET, which returns redis.Nil) — no special not-found handling needed.
	vals, err := r.client.LRange(ctx, key, 0, -1).Result()
	if err != nil {
		return nil, fmt.Errorf("failed to lrange redis key=%s: %w", key, err)
	}

	var result []*diagnostic.Diagnostic
	for _, val := range vals {
		var entry DiagnosticEntry
		if err := json.Unmarshal([]byte(val), &entry); err != nil {
			reearth_log.Warnfc(ctx, "redis: failed to unmarshal diagnostic entry: %s", val)
			continue
		}

		domainDiagnostic, err := entry.ToDomain()
		if err != nil {
			reearth_log.Warnfc(ctx, "redis: failed to convert diagnostic entry to domain: %v", err)
			continue
		}

		result = append(result, domainDiagnostic)
	}

	return result, nil
}
