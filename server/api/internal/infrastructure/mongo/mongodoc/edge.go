package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/edge"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type EdgeExecutionDocument struct {
	ID                  string     `bson:"id"`
	EdgeID              string     `bson:"edgeId"`
	JobID               string     `bson:"jobId"`
	Status              string     `bson:"status"`
	StartedAt           *time.Time `bson:"startedAt,omitempty"`
	CompletedAt         *time.Time `bson:"completedAt,omitempty"`
	FeatureID           *string    `bson:"featureId,omitempty"`
	IntermediateDataURL *string    `bson:"intermediateDataUrl,omitempty"`
}

type EdgeExecutionConsumer = Consumer[*EdgeExecutionDocument, *edge.EdgeExecution]

func NewEdgeExecutionConsumer() *EdgeExecutionConsumer {
	return NewConsumer[*EdgeExecutionDocument](func(a *edge.EdgeExecution) bool {
		return true
	})
}

func (d *EdgeExecutionDocument) Model() (*edge.EdgeExecution, error) {
	if d == nil {
		return nil, nil
	}

	jobID, err := id.JobIDFrom(d.JobID)
	if err != nil {
		return nil, err
	}

	return edge.New().
		ID(d.ID).
		EdgeID(d.EdgeID).
		JobID(jobID).
		Status(edge.Status(d.Status)).
		StartedAt(d.StartedAt).
		CompletedAt(d.CompletedAt).
		FeatureID(d.FeatureID).
		IntermediateDataURL(d.IntermediateDataURL).
		Build()
}
