package mongo

import (
	"context"
	"fmt"
	"log"

	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
)

// diagnosticIndexKeys declares the nodeDiagnostics indexes: compound
// {jobId,nodeId} plus {jobId}; no unique index since rows are appended, never upserted.
var diagnosticIndexKeys = []string{"jobId,nodeId", "jobId"}

// Init ensures the nodeDiagnostics collection has the indexes the api-side
// read path relies on. Safe to call repeatedly (mongox reconciles declaratively).
func (m *MongoStorage) Init(ctx context.Context) error {
	_, _, err := m.diagnosticsClient.Indexes(ctx, diagnosticIndexKeys, nil)
	return err
}

// SaveDiagnosticToMongo appends a DiagnosticDocument row; diagnostics are
// append-only (unlike SaveNodeExecutionToMongo), so every event gets its own row.
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
