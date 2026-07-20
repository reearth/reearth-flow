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

func ToNodeExecutions(nodes []*graph.NodeExecution) []*NodeExecution {
	if nodes == nil {
		return nil
	}

	result := make([]*NodeExecution, 0, len(nodes))
	for _, n := range nodes {
		if ne := ToNodeExecution(n); ne != nil {
			result = append(result, ne)
		}
	}
	return result
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
