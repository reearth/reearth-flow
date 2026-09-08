package mongo

import (
	"context"
	"fmt"
	"log"

	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
	"github.com/reearth/reearthx/mongox"
)

var diagnosticIndexKeys = []string{"jobId,nodeId", "jobId"}

type MongoStorage struct {
	diagnosticsClient *mongox.ClientCollection
}

func NewMongoStorage(client *mongox.Client) *MongoStorage {
	return &MongoStorage{diagnosticsClient: client.WithCollection("nodeDiagnostics")}
}

func (m *MongoStorage) Init(ctx context.Context) error {
	_, _, err := m.diagnosticsClient.Indexes(ctx, diagnosticIndexKeys, nil)
	return err
}

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
