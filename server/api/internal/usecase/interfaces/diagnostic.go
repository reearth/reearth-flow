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
	// (GraphQL Job.failedNodes). Unlike GetJobDiagnostics, reads Mongo only,
	// never Redis: these rows are persisted exclusively at job-completion
	// merge time. Filtered to EffectiveDisposition == "fatal", which the
	// engine guarantees for every failedNodes entry (never for
	// aggregatedDiagnostics entries).
	GetFailedNodes(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error)
	// GetDroppedEventCount returns the job's persisted droppedEventCount
	// (GraphQL Job.droppedEventCount). nil, nil means no summary row exists.
	GetDroppedEventCount(ctx context.Context, jobID id.JobID) (*uint64, error)
}
