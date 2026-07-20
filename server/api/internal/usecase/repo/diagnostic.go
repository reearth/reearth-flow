package repo

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

// NodeDiagnostics is the durable (Mongo) store for structured diagnostics,
// backed by the nodeDiagnostics collection.
//
// FindByJobNodeID / FindByJobID read rows written by the subscriber's
// DiagnosticEvent ingestion (server/subscriber's diagnostic Mongo writer)
// plus any terminal failed-node rows this repo itself writes via
// SaveTerminalDiagnostics — both use the same DiagnosticDocument shape, so
// they compose. SaveTerminalDiagnostics is the write-side counterpart used
// ONLY at job-completion merge time (interactor/job.go) to snapshot a
// JobCompleteEvent's FailedNodes/AggregatedDiagnostics/DroppedEventCount
// before the source event is deleted from Redis.
type NodeDiagnostics interface {
	FindByJobNodeID(ctx context.Context, jobID id.JobID, nodeID string) ([]*diagnostic.Diagnostic, error)
	FindByJobID(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error)
	// FindJobSummary reads the single per-job summary row written by
	// SaveTerminalDiagnostics (deterministic ID {jobId}:_job:summary, see
	// mongodoc.JobDiagnosticsSummaryDocument), returning its
	// droppedEventCount. Returns (nil, nil) when no summary row exists.
	FindJobSummary(ctx context.Context, jobID id.JobID) (*uint64, error)
	// SaveTerminalDiagnostics upserts one row per failedNode (deterministic
	// ID {jobId}:{nodeId-or-_job}:failed:{code}) and one row per aggregated
	// diagnostic (deterministic ID {jobId}:{nodeId-or-_job}:aggregated:
	// {code}) — both idempotent across JobCompleteEvent redeliveries — plus,
	// when a droppedEventCount is present, a single per-job summary row
	// (deterministic ID {jobId}:_job:summary). timestamp is the source
	// JobCompleteEvent's timestamp, applied to every row written.
	SaveTerminalDiagnostics(
		ctx context.Context,
		jobID id.JobID,
		timestamp time.Time,
		failedNodes []*diagnostic.Diagnostic,
		aggregated []*diagnostic.Diagnostic,
		droppedEventCount *uint64,
	) error
}
