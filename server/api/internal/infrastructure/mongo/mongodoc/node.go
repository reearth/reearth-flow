package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type NodeExecutionDocument struct {
	StartedAt   *time.Time `bson:"startedAt,omitempty"`
	CompletedAt *time.Time `bson:"completedAt,omitempty"`
	ID          string     `bson:"id"`
	JobID       string     `bson:"jobId"`
	NodeID      string     `bson:"nodeId"`
	Status      string     `bson:"status"`
}

type NodeExecutionConsumer = Consumer[*NodeExecutionDocument, *graph.NodeExecution]

func NewNodeExecutionConsumer() *NodeExecutionConsumer {
	return NewConsumer[*NodeExecutionDocument](func(a *graph.NodeExecution) bool {
		return true
	})
}

func (d *NodeExecutionDocument) Model() (*graph.NodeExecution, error) {
	if d == nil {
		return nil, nil
	}

	jobID, err := id.JobIDFrom(d.JobID)
	if err != nil {
		return nil, err
	}

	nodeID, err := id.NodeIDFrom(d.NodeID)
	if err != nil {
		return nil, err
	}

	return graph.NewNodeExecutionBuilder().
		ID(d.ID).
		JobID(jobID).
		NodeID(nodeID).
		Status(graph.Status(d.Status)).
		StartedAt(d.StartedAt).
		CompletedAt(d.CompletedAt).
		Build()
}
