package mongo

import (
	"context"
	"fmt"
	"log"

	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
)

// diagnosticIndexKeys declares the nodeDiagnostics collection's indexes: a
// compound {jobId,nodeId} index for per-node reads and a {jobId} index for
// whole-job reads (mirrors api/internal/infrastructure/mongo/node.go's
// index declarations). Diagnostic rows are appended, never upserted, so
// there is no unique index.
var diagnosticIndexKeys = []string{"jobId,nodeId", "jobId"}

// Init ensures the nodeDiagnostics collection has the indexes the api-side
// read path relies on. Safe to call repeatedly: reearthx/mongox.Collection
// declaratively reconciles indexes against what already exists.
func (m *MongoStorage) Init(ctx context.Context) error {
	_, _, err := m.diagnosticsClient.Indexes(ctx, diagnosticIndexKeys, nil)
	return err
}

// SaveDiagnosticToMongo appends a DiagnosticDocument row to the
// nodeDiagnostics collection. Diagnostics are append-only (unlike
// SaveNodeExecutionToMongo's find-then-update-or-insert): every event gets
// its own row.
func (m *MongoStorage) SaveDiagnosticToMongo(ctx context.Context, event *diagnostic.DiagnosticEvent) error {
	if event == nil {
		log.Printf("ERROR: Attempted to save nil diagnostic event to MongoDB")
		return fmt.Errorf("diagnostic event is nil")
	}

	doc := mongodoc.NewDiagnosticDocument(event)

	if err := m.diagnosticsClient.SaveOne(ctx, doc.ID, doc); err != nil {
		log.Printf("ERROR: Failed to save diagnostic event to MongoDB for jobID=%s, nodeID=%v: %v",
			event.JobID, event.NodeID, err)
		return fmt.Errorf("failed to save diagnostic event: %w", err)
	}

	return nil
}
