package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/subscriber/pkg/node"
	"go.mongodb.org/mongo-driver/bson"
)

// NodeExecutionDocument mirrors the api-side `mongo/mongodoc.NodeExecutionDocument`
// for the shared `nodeExecutions` collection: the metrics fields are stored
// flat (not nested under a `metrics` sub-document), matching that side's
// existing startedAt/completedAt shape.
type NodeExecutionDocument struct {
	ID                 string     `bson:"id"`
	JobID              string     `bson:"jobId"`
	NodeID             string     `bson:"nodeId"`
	Status             string     `bson:"status"`
	StartedAt          *time.Time `bson:"startedAt,omitempty"`
	CompletedAt        *time.Time `bson:"completedAt,omitempty"`
	FeaturesProcessed  *uint64    `bson:"featuresProcessed,omitempty"`
	FeaturesWritten    *uint64    `bson:"featuresWritten,omitempty"`
	FinishFeatureCount *uint64    `bson:"finishFeatureCount,omitempty"`
}

func NewNodeExecution(n *node.NodeExecution) NodeExecutionDocument {
	doc := NodeExecutionDocument{
		ID:          n.ID,
		JobID:       n.JobID,
		NodeID:      n.NodeID,
		Status:      string(n.Status),
		StartedAt:   n.StartedAt,
		CompletedAt: n.CompletedAt,
	}
	if n.Metrics != nil {
		doc.FeaturesProcessed = &n.Metrics.FeaturesProcessed
		doc.FeaturesWritten = &n.Metrics.FeaturesWritten
		doc.FinishFeatureCount = &n.Metrics.FinishFeatureCount
	}
	return doc
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

	var metrics *node.NodeMetrics
	if doc.FeaturesProcessed != nil || doc.FeaturesWritten != nil || doc.FinishFeatureCount != nil {
		m := node.NodeMetrics{}
		if doc.FeaturesProcessed != nil {
			m.FeaturesProcessed = *doc.FeaturesProcessed
		}
		if doc.FeaturesWritten != nil {
			m.FeaturesWritten = *doc.FeaturesWritten
		}
		if doc.FinishFeatureCount != nil {
			m.FinishFeatureCount = *doc.FinishFeatureCount
		}
		metrics = &m
	}

	c.Result = append(c.Result, &node.NodeExecution{
		ID:          doc.ID,
		JobID:       doc.JobID,
		NodeID:      doc.NodeID,
		Status:      node.Status(doc.Status),
		StartedAt:   doc.StartedAt,
		CompletedAt: doc.CompletedAt,
		Metrics:     metrics,
	})

	return nil
}
