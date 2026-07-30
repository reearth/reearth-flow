package postgres

import (
	"context"
	"fmt"
	"log"

	"github.com/reearth/reearth-flow/subscriber/pkg/node"
	"github.com/reearth/reearthx/pgxx"
)

// The node execution ID is jobID:nodeID, so the primary key carries the
// per-node uniqueness the Mongo path enforced with a jobId+nodeId filter.
const upsertNodeExecution = `INSERT INTO node_executions (
  id, job_id, node_id, status, started_at, completed_at
) VALUES ($1, $2, $3, $4, $5, $6)
ON CONFLICT (id) DO UPDATE SET
  job_id       = EXCLUDED.job_id,
  node_id      = EXCLUDED.node_id,
  status       = EXCLUDED.status,
  started_at   = EXCLUDED.started_at,
  completed_at = EXCLUDED.completed_at`

type PostgresStorage struct {
	c *pgxx.Client
}

func NewPostgresStorage(c *pgxx.Client) *PostgresStorage {
	return &PostgresStorage{c: c}
}

func (p *PostgresStorage) SaveNodeExecution(ctx context.Context, jobID string, nodeExec *node.NodeExecution) error {
	if nodeExec == nil {
		log.Printf("ERROR: Attempted to save nil node execution for jobID=%s", jobID)
		return fmt.Errorf("node execution is nil")
	}

	log.Printf("DEBUG: Saving node execution to Postgres for jobID=%s, nodeID=%s, status=%s",
		jobID, nodeExec.NodeID, nodeExec.Status)

	if _, err := p.c.DB(ctx).Exec(ctx, upsertNodeExecution,
		nodeExec.ID,
		jobID,
		nodeExec.NodeID,
		string(nodeExec.Status),
		nodeExec.StartedAt,
		nodeExec.CompletedAt,
	); err != nil {
		log.Printf("ERROR: Failed to save node execution: %v", err)
		return fmt.Errorf("failed to save node execution: %w", pgxx.WrapError(err))
	}

	return nil
}
