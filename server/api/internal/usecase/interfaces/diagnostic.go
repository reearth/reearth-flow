package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type NodeDiagnostics interface {
	GetNodeDiagnostics(ctx context.Context, jobID id.JobID, nodeID string) ([]*diagnostic.Diagnostic, error)
	GetJobDiagnostics(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error)
	// GetFailedNodes returns the job's terminal per-node fatal-failure rows
	// (GraphQL Job.failedNodes). Unlike GetJobDiagnostics, this reads Mongo
	// ONLY, never Redis: failedNodes/aggregatedDiagnostics rows are persisted
	// exclusively at job-completion merge time (interactor/job.go's
	// persistTerminalDiagnostics) and never written to Redis, so blending in
	// Redis's live per-event diagnostics here would misrepresent unrelated
	// in-flight rows as job failures. The result is filtered to rows whose
	// EffectiveDisposition is the engine's authoritative "fatal" value,
	// which is exactly (and only) the set that originated from the
	// JobCompleteEvent's failedNodes wire array: the engine guarantees every
	// failedNodes entry is stamped effective_disposition=Fatal
	// (dag_executor.rs's fold_outcomes) and that aggregatedDiagnostics
	// entries are never Fatal (job_complete_event.json's field contract).
	GetFailedNodes(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error)
	// GetDroppedEventCount returns the job's persisted droppedEventCount
	// (GraphQL Job.droppedEventCount), read from the single per-job summary
	// row persisted alongside failedNodes/aggregatedDiagnostics at job-
	// completion merge time. nil, nil means no summary row exists (the
	// JobCompleteEvent never carried a droppedEventCount, e.g. old-wire
	// events or jobs with nothing dropped).
	GetDroppedEventCount(ctx context.Context, jobID id.JobID) (*uint64, error)
}
