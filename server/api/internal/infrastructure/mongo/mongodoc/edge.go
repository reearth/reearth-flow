package mongodoc

import (
	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

// EdgeExecutionDocument decodes the legacy "edgeExecutions" collection for
// cmd/dbmigrate's Mongo->Postgres ETL. No Mongo repo reads it.
type EdgeExecutionDocument struct {
	IntermediateDataURL *string `bson:"intermediateDataUrl,omitempty"`
	ID                  string  `bson:"id"`
	EdgeID              string  `bson:"edgeId"`
	JobID               string  `bson:"jobId"`
}

func (d *EdgeExecutionDocument) Model() (*graph.EdgeExecution, error) {
	if d == nil {
		return nil, nil
	}

	eeid, err := id.EdgeExecutionIDFrom(d.ID)
	if err != nil {
		return nil, err
	}

	jobID, err := id.JobIDFrom(d.JobID)
	if err != nil {
		return nil, err
	}

	return graph.NewEdgeExecutionBuilder().
		ID(eeid).
		EdgeID(d.EdgeID).
		JobID(jobID).
		IntermediateDataURL(d.IntermediateDataURL).
		Build()
}
