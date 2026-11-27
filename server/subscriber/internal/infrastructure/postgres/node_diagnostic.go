package postgres

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"strings"

	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
)

// Same segment the Mongo writer and the API use for job-level row IDs.
const jobDiagnosticNodeSegment = "_job"

// Pub/Sub redelivers the same bytes, so the payload-hashed ID makes a redelivery a no-op.
const insertNodeDiagnosticSQL = `INSERT INTO node_diagnostics (
  id, job_id, workflow_id, node_id, ts, schema, code, category, severity, message,
  effective_disposition, action_type, feature_id, help,
  source_span_offset, source_span_length, aggregated_count, aggregated_sample_feature_ids
) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
ON CONFLICT (id) DO NOTHING`

// NodeDiagnostics writes the durable node_diagnostics rows the API reads, replacing the Mongo nodeDiagnostics collection.
type NodeDiagnostics struct {
	q Querier
}

func NewNodeDiagnostics(q Querier) *NodeDiagnostics {
	return &NodeDiagnostics{q: q}
}

func (s *NodeDiagnostics) SaveDurableDiagnostic(ctx context.Context, event *diagnostic.DiagnosticEvent) error {
	if event == nil {
		return errors.New("cannot save nil diagnostic event")
	}
	serialized, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("failed to marshal diagnostic event: %w", err)
	}
	hash := sha256.Sum256(serialized)

	nodeID := nullIfEmpty(event.NodeID)
	nodeSegment := jobDiagnosticNodeSegment
	if nodeID != nil {
		nodeSegment = *nodeID
	}

	var (
		spanOffset, spanLength, aggCount *int64
		sampleIDs                        []string
	)
	if ss := event.SourceSpan; ss != nil {
		o := int64(ss.Offset) //nolint:gosec
		spanOffset = &o
		if ss.Length != nil {
			l := int64(*ss.Length) //nolint:gosec
			spanLength = &l
		}
	}
	if agg := event.Aggregated; agg != nil {
		c := int64(agg.Count) //nolint:gosec
		aggCount = &c
		sampleIDs = agg.SampleFeatureIds
	}

	if _, err := s.q.Exec(ctx, insertNodeDiagnosticSQL,
		event.JobID+":"+nodeSegment+":"+hex.EncodeToString(hash[:16]),
		event.JobID, event.WorkflowID, nodeID, event.Timestamp.UTC(), event.Schema,
		event.Code, event.Category, event.Severity, []byte(event.Message),
		event.EffectiveDisposition, event.ActionType, event.FeatureID, event.Help,
		spanOffset, spanLength, aggCount, sampleIDs,
	); err != nil {
		return fmt.Errorf("failed to insert node diagnostic: %w", err)
	}
	return nil
}

// Ping reports whether node_diagnostics exists and the current user can insert into it.
func (s *NodeDiagnostics) Ping(ctx context.Context) error {
	var problems []string
	if err := s.q.QueryRow(ctx, readySQL, []string{"node_diagnostics"}, []string{"INSERT"}).Scan(&problems); err != nil {
		return fmt.Errorf("check node_diagnostics: %w", err)
	}
	if len(problems) > 0 {
		return fmt.Errorf("node_diagnostics not ready: %s", strings.Join(problems, "; "))
	}
	return nil
}
