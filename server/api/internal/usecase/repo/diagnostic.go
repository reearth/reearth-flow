package repo

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type NodeDiagnostics interface {
	// Empty nodeID reads the job-level bucket.
	FindByJobNodeID(ctx context.Context, jobID id.JobID, nodeID string) ([]*diagnostic.Diagnostic, error)
	FindByJobID(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error)
	// Returns (nil, nil) when no summary row exists.
	FindJobSummary(ctx context.Context, jobID id.JobID) (*uint64, error)
	// Idempotent across redeliveries: the row IDs are deterministic.
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
