package interactor

import (
	"context"
	"fmt"
	"log"
	"time"

	"github.com/reearth/reearth-flow/subscriber/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/subscriber/pkg/edge"
	"github.com/reearth/reearth-flow/subscriber/pkg/id"
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
		log.Printf("ERROR: Received nil event")
		return fmt.Errorf("event is nil")
	}

	log.Printf("DEBUG: Processing edge event for JobID: %s with %d updated edges",
		event.JobID, len(event.UpdatedEdges))

	if err := u.storage.SaveToRedis(ctx, event); err != nil {
		log.Printf("ERROR: Failed to save event to Redis for JobID %s: %v", event.JobID, err)
		return fmt.Errorf("failed to write to Redis: %w", err)
	}
	log.Printf("DEBUG: Successfully saved event to Redis for JobID: %s", event.JobID)

	for i, updatedEdge := range event.UpdatedEdges {
		log.Printf("DEBUG: Processing updated edge %d/%d: ID=%s, Status=%s",
			i+1, len(event.UpdatedEdges), updatedEdge.ID, updatedEdge.Status)

		var featureIDStr *string
		if updatedEdge.FeatureID != nil {
			featureIDStr = updatedEdge.FeatureID
			log.Printf("DEBUG: Edge %s has FeatureID: %s", updatedEdge.ID, *featureIDStr)
		} else {
			log.Printf("DEBUG: Edge %s has no FeatureID", updatedEdge.ID)
		}

		edgeExec := &edge.EdgeExecution{
			ID:        id.NewEdgeExecutionID().String(),
			EdgeID:    updatedEdge.ID,
			Status:    updatedEdge.Status,
			FeatureID: featureIDStr,
		}

		now := time.Now()
		if updatedEdge.Status == edge.StatusInProgress {
			edgeExec.StartedAt = &now
			log.Printf("DEBUG: Setting StartedAt=%s for edge %s", now.Format(time.RFC3339), updatedEdge.ID)
		} else if updatedEdge.Status == edge.StatusCompleted {
			edgeExec.CompletedAt = &now
			log.Printf("DEBUG: Setting CompletedAt=%s for edge %s", now.Format(time.RFC3339), updatedEdge.ID)
		}

		intermediateDataURL := u.storage.ConstructIntermediateDataURL(event.JobID, updatedEdge.ID)
		edgeExec.IntermediateDataURL = intermediateDataURL
		log.Printf("DEBUG: Constructed IntermediateDataURL: %s", intermediateDataURL)

		if err := u.storage.UpdateEdgeStatusInMongo(ctx, event.JobID, edgeExec); err != nil {
			log.Printf("WARNING: Failed to update edge status in MongoDB for JobID=%s, EdgeID=%s: %v",
				event.JobID, updatedEdge.ID, err)
		} else {
			log.Printf("DEBUG: Successfully updated edge status in MongoDB for JobID=%s, EdgeID=%s",
				event.JobID, updatedEdge.ID)
		}
	}

	log.Printf("DEBUG: Successfully processed all edges for JobID: %s", event.JobID)
	return nil
}
