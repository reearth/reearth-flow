package interactor

import (
	"context"
	"fmt"
	"log"
	"time"

	"github.com/reearth/reearth-flow/subscriber/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/subscriber/pkg/node"
)

type NodeSubscriberUseCase interface {
	ProcessNodeEvent(ctx context.Context, event *node.NodeStatusEvent) error
}

type nodeSubscriberUseCase struct {
	storage gateway.NodeStorage
}

func NewNodeSubscriberUseCase(storage gateway.NodeStorage) NodeSubscriberUseCase {
	return &nodeSubscriberUseCase{
		storage: storage,
	}
}

func (u *nodeSubscriberUseCase) ProcessNodeEvent(ctx context.Context, event *node.NodeStatusEvent) error {
	if event == nil {
		log.Printf("ERROR: Received nil node event")
		return fmt.Errorf("event is nil")
	}

	log.Printf("DEBUG: Processing node event for JobID: %s, NodeID: %s with status %s",
		event.JobID, event.NodeID, event.Status)

	if err := u.storage.SaveToRedis(ctx, event); err != nil {
		log.Printf("ERROR: Failed to save node event to Redis for JobID %s: %v", event.JobID, err)
		return fmt.Errorf("failed to write to Redis: %w", err)
	}
	log.Printf("DEBUG: Successfully saved node event to Redis for JobID: %s, NodeID: %s", event.JobID, event.NodeID)

	if event.Status == node.StatusCompleted || event.Status == node.StatusFailed {
		nodeExecID := fmt.Sprintf("%s-%s", event.JobID, event.NodeID)

		nodeExec := &node.NodeExecution{
			ID:     nodeExecID,
			JobID:  event.JobID,
			NodeID: event.NodeID,
			Status: event.Status,
		}

		now := time.Now()
		nodeExec.CompletedAt = &now
		log.Printf("DEBUG: Setting CompletedAt=%s for node %s", now.Format(time.RFC3339), event.NodeID)

		if err := u.storage.SaveToMongo(ctx, event.JobID, nodeExec); err != nil {
			log.Printf("WARNING: Failed to save node execution to MongoDB for JobID=%s, NodeID=%s: %v",
				event.JobID, event.NodeID, err)
		} else {
			log.Printf("DEBUG: Successfully saved node execution to MongoDB for JobID=%s, NodeID=%s",
				event.JobID, event.NodeID)
		}
	} else {
		log.Printf("DEBUG: Skipping MongoDB save for non-terminal status %s", event.Status)
	}

	log.Printf("DEBUG: Successfully processed node event for JobID: %s, NodeID: %s", event.JobID, event.NodeID)
	return nil
}
