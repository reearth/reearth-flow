package postgres

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/gen"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
)

type EdgeExecution struct {
	c *pgxx.Client
}

var _ repo.EdgeExecution = (*EdgeExecution)(nil)

func NewEdgeExecution(c *pgxx.Client) *EdgeExecution {
	return &EdgeExecution{c: c}
}

func (r *EdgeExecution) q(ctx context.Context) *gen.Queries {
	return gen.New(r.c.DB(ctx))
}

func (r *EdgeExecution) FindByJobEdgeID(ctx context.Context, jobID id.JobID, edgeID string) (*graph.EdgeExecution, error) {
	row, err := r.q(ctx).GetEdgeExecutionByJobEdgeID(ctx, gen.GetEdgeExecutionByJobEdgeIDParams{
		JobID:  jobID.String(),
		EdgeID: edgeID,
	})
	if err != nil {
		return nil, pgxx.MapError(err)
	}
	return edgeExecutionFromRow(row)
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

func edgeExecutionFromRow(row gen.EdgeExecution) (*graph.EdgeExecution, error) {
	eid, err := id.EdgeExecutionIDFrom(row.ID)
	if err != nil {
		return nil, err
	}
	jid, err := id.JobIDFrom(row.JobID)
	if err != nil {
		return nil, err
	}
	return graph.NewEdgeExecutionBuilder().
		ID(eid).
		EdgeID(row.EdgeID).
		JobID(jid).
		IntermediateDataURL(row.IntermediateDataUrl).
		Build()
}
