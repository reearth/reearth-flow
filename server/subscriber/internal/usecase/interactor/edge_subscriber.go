package interactor

import (
	"context"
	"fmt"

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

		existingEdgeExec, _ := u.storage.FindEdgeExecution(ctx, event.JobID, updatedEdge.ID)

		var edgeExec *edge.EdgeExecution
		now := time.Now()

		if existingEdgeExec == nil {
			edgeExec = &edge.EdgeExecution{
				ID:        id.NewEdgeExecutionID().String(),
				EdgeID:    updatedEdge.ID,
				Status:    updatedEdge.Status,
				FeatureID: featureIDStr,
			}

			if updatedEdge.Status == edge.StatusInProgress {
				edgeExec.StartedAt = &now
			} else if updatedEdge.Status == edge.StatusCompleted {
				edgeExec.CompletedAt = &now
			}
		} else {
			edgeExec = existingEdgeExec
			edgeExec.Status = updatedEdge.Status

			if featureIDStr != nil {
				edgeExec.FeatureID = featureIDStr
			}

			if updatedEdge.Status == edge.StatusInProgress && edgeExec.StartedAt == nil {
				edgeExec.StartedAt = &now
			}

			if updatedEdge.Status == edge.StatusCompleted {
				edgeExec.CompletedAt = &now
			}
		}

		intermediateDataURL := u.storage.ConstructIntermediateDataURL(event.JobID, updatedEdge.ID)
		edgeExec.IntermediateDataURL = intermediateDataURL

		_ = u.storage.UpdateEdgeStatusInMongo(ctx, event.JobID, edgeExec)
	}

	return nil
}
