package postgres

import (
	"context"
	"fmt"
	"log"

	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/postgres/gen"
	"github.com/reearth/reearth-flow/subscriber/pkg/node"
	"github.com/reearth/reearthx/pgxx"
)

type PostgresStorage struct {
	c *pgxx.Client
}

func NewPostgresStorage(c *pgxx.Client) *PostgresStorage {
	return &PostgresStorage{c: c}
}

func (p *PostgresStorage) q(ctx context.Context) *gen.Queries {
	return gen.New(p.c.DB(ctx))
}

func (p *PostgresStorage) SaveNodeExecution(ctx context.Context, jobID string, nodeExec *node.NodeExecution) error {
	if nodeExec == nil {
		log.Printf("ERROR: Attempted to save nil node execution for jobID=%s", jobID)
		return fmt.Errorf("node execution is nil")
	}

	log.Printf("DEBUG: Saving node execution to Postgres for jobID=%s, nodeID=%s, status=%s",
		jobID, nodeExec.NodeID, nodeExec.Status)

	if err := p.q(ctx).UpsertNodeExecution(ctx, gen.UpsertNodeExecutionParams{
		ID:          nodeExec.ID,
		JobID:       jobID,
		NodeID:      nodeExec.NodeID,
		Status:      string(nodeExec.Status),
		StartedAt:   nodeExec.StartedAt,
		CompletedAt: nodeExec.CompletedAt,
	}); err != nil {
		log.Printf("ERROR: Failed to save node execution: %v", err)
		return fmt.Errorf("failed to save node execution: %w", pgxx.WrapError(err))
	}

	return nil
}
