package postgres

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/gen"
	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
)

// EdgeExecution writes edge_executions rows. Kept for cmd/dbmigrate's
// Mongo->Postgres ETL; there is no read path (nothing in the API reads
// this table).
type EdgeExecution struct {
	c *pgxx.Client
}

func NewEdgeExecution(c *pgxx.Client) *EdgeExecution {
	return &EdgeExecution{c: c}
}

func (r *EdgeExecution) q(ctx context.Context) *gen.Queries {
	return gen.New(r.c.DB(ctx))
}

func (r *EdgeExecution) Save(ctx context.Context, e *graph.EdgeExecution) error {
	if err := r.q(ctx).UpsertEdgeExecution(ctx, gen.UpsertEdgeExecutionParams{
		ID:                  e.ID().String(),
		EdgeID:              e.EdgeID(),
		JobID:               e.JobID().String(),
		IntermediateDataUrl: e.IntermediateDataURL(),
	}); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}
