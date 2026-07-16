package interactor

import (
	"context"
	"fmt"
	"log"

	"github.com/reearth/reearth-flow/subscriber/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
)

type DiagnosticSubscriberUseCase interface {
	ProcessDiagnosticEvent(ctx context.Context, event *diagnostic.DiagnosticEvent) error
}

type diagnosticSubscriberUseCase struct {
	storage gateway.DiagnosticStorage
}

func NewDiagnosticSubscriberUseCase(storage gateway.DiagnosticStorage) DiagnosticSubscriberUseCase {
	return &diagnosticSubscriberUseCase{
		storage: storage,
	}
}

func (u *diagnosticSubscriberUseCase) ProcessDiagnosticEvent(ctx context.Context, event *diagnostic.DiagnosticEvent) error {
	if event == nil {
		return fmt.Errorf("diagnostic event is nil")
	}

	if event.Schema != diagnostic.DiagnosticSchemaV1 {
		return fmt.Errorf("invalid event: unexpected schema %q (expected %q)", event.Schema, diagnostic.DiagnosticSchemaV1)
	}

	if event.JobID == "" {
		return fmt.Errorf("invalid event: missing job ID")
	}

	if err := u.storage.SaveToRedis(ctx, event); err != nil {
		return fmt.Errorf("failed to write diagnostic event to Redis: %w", err)
	}

	// Mongo persistence is best-effort: Redis is the source of truth for
	// live consumption, Mongo is the durable per-node record. A Mongo
	// failure is logged and swallowed so the message is still Acked
	// (mirrors node_subscriber.go's terminal-state Mongo write).
	if err := u.storage.SaveToMongo(ctx, event); err != nil {
		log.Printf("WARNING: Failed to save diagnostic event to MongoDB for JobID=%s, NodeID=%v: %v",
			event.JobID, event.NodeID, err)
	}

	return nil
}
