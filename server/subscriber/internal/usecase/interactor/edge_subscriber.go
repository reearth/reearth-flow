package interactor

import (
	"context"
	"fmt"
	"log"
	"time"

	"github.com/reearth/reearth-flow/subscriber/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/subscriber/pkg/edge"
)

type EdgeSubscriberUseCase interface {
	ProcessEdgeEvent(ctx context.Context, event *edge.PassThroughEvent) error
}

type edgeSubscriberUseCase struct {
	storage gateway.EdgeStorage
}

func NewEdgeSubscriberUseCase(storage gateway.EdgeStorage) EdgeSubscriberUseCase {
	return &edgeSubscriberUseCase{
		storage: storage,
	}
}

func (u *edgeSubscriberUseCase) ProcessEdgeEvent(ctx context.Context, event *edge.PassThroughEvent) error {
	if event == nil {
		return fmt.Errorf("event is nil")
	}

	if err := u.storage.SaveToRedis(ctx, event); err != nil {
		return fmt.Errorf("failed to write to Redis: %w", err)
	}

	for _, updatedEdge := range event.UpdatedEdges {
		var featureIDStr *string
		if updatedEdge.FeatureID != nil {
			featureIDStr = updatedEdge.FeatureID
		}

		edgeExec := &edge.EdgeExecution{
			ID:        updatedEdge.ID,
			Status:    updatedEdge.Status,
			FeatureID: featureIDStr,
		}

		now := time.Now()

		if updatedEdge.Status == edge.StatusInProgress {
			edgeExec.StartedAt = &now
		} else if updatedEdge.Status == edge.StatusCompleted {
			edgeExec.CompletedAt = &now
		}

		edgeExec.IntermediateDataURL = u.storage.ConstructIntermediateDataURL(event.JobID, updatedEdge.ID)

		if err := u.storage.UpdateEdgeStatusInMongo(ctx, event.JobID, edgeExec); err != nil {
			log.Printf("Warning: Failed to update edge status in MongoDB: %v", err)
		}
	}

	return nil
}
