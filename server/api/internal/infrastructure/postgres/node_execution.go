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

type NodeExecution struct {
	c *pgxx.Client
}

var _ repo.NodeExecution = (*NodeExecution)(nil)

func NewNodeExecution(c *pgxx.Client) *NodeExecution {
	return &NodeExecution{c: c}
}

func (r *NodeExecution) q(ctx context.Context) *gen.Queries {
	return gen.New(r.c.DB(ctx))
}

func (r *NodeExecution) FindByJobNodeID(ctx context.Context, jobID id.JobID, nodeID string) (*graph.NodeExecution, error) {
	row, err := r.q(ctx).GetNodeExecutionByJobNodeID(ctx, gen.GetNodeExecutionByJobNodeIDParams{
		JobID:  jobID.String(),
		NodeID: nodeID,
	})
	if err != nil {
		return nil, pgxx.MapError(err)
	}
	return nodeExecutionFromRow(row)
}

// Save persists a NodeExecution. It is not part of the repo.NodeExecution
// interface (writes come from the subscriber), but is exported so tests and
// the subscriber adapter can use it directly.
func (r *NodeExecution) Save(ctx context.Context, e *graph.NodeExecution) error {
	if err := r.q(ctx).UpsertNodeExecution(ctx, gen.UpsertNodeExecutionParams{
		ID:          e.ID(),
		JobID:       e.JobID().String(),
		NodeID:      e.NodeID().String(),
		Status:      string(e.Status()),
		StartedAt:   e.StartedAt(),
		CompletedAt: e.CompletedAt(),
	}); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func nodeExecutionFromRow(row gen.NodeExecution) (*graph.NodeExecution, error) {
	jid, err := id.JobIDFrom(row.JobID)
	if err != nil {
		return nil, err
	}
	nid, err := id.NodeIDFrom(row.NodeID)
	if err != nil {
		return nil, err
	}
	return graph.NewNodeExecutionBuilder().
		ID(row.ID).
		JobID(jid).
		NodeID(nid).
		Status(graph.Status(row.Status)).
		StartedAt(row.StartedAt).
		CompletedAt(row.CompletedAt).
		Build()
}
