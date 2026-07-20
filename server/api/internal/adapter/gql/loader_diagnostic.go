package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

// DiagnosticLoader wraps interfaces.NodeDiagnostics for the GraphQL layer,
// mirroring the thin non-batching wrapper pattern used by LogLoader/
// NodeExLoader (there is no per-key batching to do here: each of the three
// methods below backs exactly one resolver field on a single parent object).
type DiagnosticLoader struct {
	usecase interfaces.NodeDiagnostics
}

func NewDiagnosticLoader(usecase interfaces.NodeDiagnostics) *DiagnosticLoader {
	return &DiagnosticLoader{usecase: usecase}
}

// GetNodeDiagnostics backs NodeExecution.diagnostics.
func (l *DiagnosticLoader) GetNodeDiagnostics(ctx context.Context, jobID gqlmodel.ID, nodeID string) ([]*gqlmodel.Diagnostic, error) {
	jId, err := id.JobIDFrom(string(jobID))
	if err != nil {
		return nil, err
	}

	rows, err := l.usecase.GetNodeDiagnostics(ctx, jId, nodeID)
	if err != nil {
		return nil, err
	}
	return gqlmodel.ToDiagnostics(rows), nil
}

// GetFailedNodes backs Job.failedNodes.
func (l *DiagnosticLoader) GetFailedNodes(ctx context.Context, jobID gqlmodel.ID) ([]*gqlmodel.Diagnostic, error) {
	jId, err := id.JobIDFrom(string(jobID))
	if err != nil {
		return nil, err
	}

	rows, err := l.usecase.GetFailedNodes(ctx, jId)
	if err != nil {
		return nil, err
	}
	return gqlmodel.ToDiagnostics(rows), nil
}

// GetDroppedEventCount backs Job.droppedEventCount.
func (l *DiagnosticLoader) GetDroppedEventCount(ctx context.Context, jobID gqlmodel.ID) (*int, error) {
	jId, err := id.JobIDFrom(string(jobID))
	if err != nil {
		return nil, err
	}

	count, err := l.usecase.GetDroppedEventCount(ctx, jId)
	if err != nil {
		return nil, err
	}
	if count == nil {
		return nil, nil
	}
	c := int(*count)
	return &c, nil
}
