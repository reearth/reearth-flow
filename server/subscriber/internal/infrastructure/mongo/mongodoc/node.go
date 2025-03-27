package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/subscriber/pkg/node"
	"go.mongodb.org/mongo-driver/bson"
)

type NodeExecutionDocument struct {
	ID          string     `bson:"id"`
	JobID       string     `bson:"jobId"`
	NodeID      string     `bson:"nodeId"`
	Status      string     `bson:"status"`
	StartedAt   *time.Time `bson:"startedAt,omitempty"`
	CompletedAt *time.Time `bson:"completedAt,omitempty"`
}

func NewNodeExecution(n *node.NodeExecution) NodeExecutionDocument {
	return NodeExecutionDocument{
		ID:          n.ID,
		JobID:       n.JobID,
		NodeID:      n.NodeID,
		Status:      string(n.Status),
		StartedAt:   n.StartedAt,
		CompletedAt: n.CompletedAt,
	}
}

type NodeExecutionConsumer struct {
	Result []*node.NodeExecution
}

func NewNodeExecutionConsumer() *NodeExecutionConsumer {
	return &NodeExecutionConsumer{
		Result: make([]*node.NodeExecution, 0),
	}
}

func (c *NodeExecutionConsumer) Consume(raw bson.Raw) error {
	if raw == nil {
		return nil
	}

	var doc NodeExecutionDocument
	if err := bson.Unmarshal(raw, &doc); err != nil {
		return err
	}

	c.Result = append(c.Result, &node.NodeExecution{
		ID:          doc.ID,
		JobID:       doc.JobID,
		NodeID:      doc.NodeID,
		Status:      node.Status(doc.Status),
		StartedAt:   doc.StartedAt,
		CompletedAt: doc.CompletedAt,
	})

	return nil
}
