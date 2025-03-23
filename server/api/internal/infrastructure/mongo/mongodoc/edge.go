package mongodoc

import (
	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/rerror"
)

type EdgeExecutionDocument struct {
	ID                  string  `bson:"id"`
	EdgeID              string  `bson:"edgeId"`
	JobID               string  `bson:"jobId"`
	IntermediateDataURL *string `bson:"intermediateDataUrl,omitempty"`
}

type EdgeExecutionConsumer = Consumer[*EdgeExecutionDocument, *graph.EdgeExecution]

func NewEdgeExecutionConsumer() *EdgeExecutionConsumer {
	return NewConsumer[*EdgeExecutionDocument](func(a *graph.EdgeExecution) bool {
		return true
	})
}

func NewEdgeExecution(e *graph.EdgeExecution) (*EdgeExecutionDocument, error) {
	if e == nil {
		return nil, rerror.ErrNotFound
	}

	eeid := e.ID().String()
	if eeid == "" {
		return nil, rerror.ErrNotFound
	}

	doc := &EdgeExecutionDocument{
		ID:                  eeid,
		EdgeID:              e.EdgeID(),
		JobID:               e.JobID().String(),
		IntermediateDataURL: e.IntermediateDataURL(),
	}

	return doc, nil
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
