package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type NodeDiagnostics interface {
	GetNodeDiagnostics(ctx context.Context, jobID id.JobID, nodeID string) ([]*diagnostic.Diagnostic, error)
	GetJobDiagnostics(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error)
	// Mongo-only, never Redis (rows are terminal, persisted only at
	// job-completion). EffectiveDisposition == "fatal" for every entry.
	GetFailedNodes(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error)
	// nil, nil means no summary row exists (not zero dropped events).
	GetDroppedEventCount(ctx context.Context, jobID id.JobID) (*uint64, error)
}
