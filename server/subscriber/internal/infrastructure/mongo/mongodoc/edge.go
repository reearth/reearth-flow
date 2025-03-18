package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/subscriber/pkg/edge"
)

type EdgeExecutionDocument struct {
	ID                  string     `bson:"id"`
	EdgeID              string     `bson:"edgeId"`
	JobID               string     `bson:"jobId"`
	Status              string     `bson:"status"`
	StartedAt           *time.Time `bson:"startedAt,omitempty"`
	CompletedAt         *time.Time `bson:"completedAt,omitempty"`
	FeatureID           *string    `bson:"featureId,omitempty"`
	IntermediateDataURL string     `bson:"intermediateDataUrl,omitempty"`
	CreatedAt           time.Time  `bson:"createdAt"`
	UpdatedAt           time.Time  `bson:"updatedAt"`
}

type EdgeExecutionConsumer = Consumer[*EdgeExecutionDocument, *edge.EdgeExecution]

func NewEdgeExecutionConsumer() *EdgeExecutionConsumer {
	return NewConsumer[*EdgeExecutionDocument](func(a *edge.EdgeExecution) bool {
		return true
	})
}

// Model converts the document to a domain model
func (d *EdgeExecutionDocument) Model() (*edge.EdgeExecution, error) {
	if d == nil {
		return nil, nil
	}

	return &edge.EdgeExecution{
		ID:                  d.ID,
		EdgeID:              d.EdgeID,
		Status:              edge.Status(d.Status),
		StartedAt:           d.StartedAt,
		CompletedAt:         d.CompletedAt,
		FeatureID:           d.FeatureID,
		IntermediateDataURL: d.IntermediateDataURL,
	}, nil
}
