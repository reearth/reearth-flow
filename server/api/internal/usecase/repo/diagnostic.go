package repo

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

// NodeDiagnostics is the durable (Mongo) store for structured diagnostics,
// backed by the nodeDiagnostics collection. Read methods return both the
// subscriber's live-ingested rows and SaveTerminalDiagnostics' terminal
// rows (same DiagnosticDocument shape); the latter is written only at
// job-completion merge time (interactor/job.go), before the source event is
// deleted from Redis.
type NodeDiagnostics interface {
	// FindByJobNodeID scopes to one node's rows. An empty nodeID reads the
	// job-level bucket, mirroring gateway.Redis.GetNodeDiagnostics' "" →
	// "_job" fallback.
	FindByJobNodeID(ctx context.Context, jobID id.JobID, nodeID string) ([]*diagnostic.Diagnostic, error)
	FindByJobID(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error)
	// FindJobSummary reads the single per-job summary row written by
	// SaveTerminalDiagnostics, returning its droppedEventCount. Returns
	// (nil, nil) when no summary row exists.
	FindJobSummary(ctx context.Context, jobID id.JobID) (*uint64, error)
	// SaveTerminalDiagnostics upserts one row per failedNode and one per
	// aggregated diagnostic (deterministic IDs, idempotent across
	// JobCompleteEvent redeliveries), plus a single per-job summary row when
	// droppedEventCount is present. timestamp is applied to every row.
	SaveTerminalDiagnostics(
		ctx context.Context,
		jobID id.JobID,
		workflowID string,
		timestamp time.Time,
		failedNodes []*diagnostic.Diagnostic,
		aggregated []*diagnostic.Diagnostic,
		droppedEventCount *uint64,
	) error
}
