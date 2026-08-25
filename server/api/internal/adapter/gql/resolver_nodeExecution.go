package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
)

type nodeExecutionResolver struct{ *Resolver }

func (r *nodeExecutionResolver) Diagnostics(ctx context.Context, obj *gqlmodel.NodeExecution) ([]*gqlmodel.Diagnostic, error) {
	return loaders(ctx).Diagnostic.GetNodeDiagnostics(ctx, obj.JobID, string(obj.NodeID))
}
