package gqlmodel

import "github.com/reearth/reearth-flow/api/pkg/graph"

func ToNodeExecution(e *graph.NodeExecution) *NodeExecution {
	if e == nil {
		return nil
	}

	return &NodeExecution{
		ID:          ID(e.ID()),
		JobID:       ID(e.JobID().String()),
		NodeID:      ID(e.NodeID().String()),
		Status:      ToNodeStatus(e.Status()),
		StartedAt:   e.StartedAt(),
		CompletedAt: e.CompletedAt(),
	}
}

func ToNodeStatus(status graph.Status) NodeStatus {
	switch status {
	case graph.StatusStarting:
		return NodeStatusStarting
	case graph.StatusProcessing:
		return NodeStatusProcessing
	case graph.StatusCompleted:
		return NodeStatusCompleted
	case graph.StatusFailed:
		return NodeStatusFailed
	default:
		return NodeStatusPending
	}
}
