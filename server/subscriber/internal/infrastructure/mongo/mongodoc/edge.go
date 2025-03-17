package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/subscriber/pkg/edge"
)

type EdgeExecutionDocument struct {
	ID                  string     `bson:"id"`
	Status              string     `bson:"status"`
	StartedAt           *time.Time `bson:"startedAt,omitempty"`
	CompletedAt         *time.Time `bson:"completedAt,omitempty"`
	FeatureID           *string    `bson:"featureId,omitempty"`
	IntermediateDataURL string     `bson:"intermediateDataUrl,omitempty"`
}

func NewEdgeExecution(e *edge.EdgeExecution) EdgeExecutionDocument {
	return EdgeExecutionDocument{
		ID:                  e.ID,
		Status:              string(e.Status),
		StartedAt:           e.StartedAt,
		CompletedAt:         e.CompletedAt,
		FeatureID:           e.FeatureID,
		IntermediateDataURL: e.IntermediateDataURL,
	}
}
