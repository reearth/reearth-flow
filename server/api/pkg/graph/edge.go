package graph

import "github.com/reearth/reearth-flow/api/pkg/id"

type EdgeExecution struct {
	intermediateDataURL *string
	edgeID              string
	id                  EdgeExecutionID
	jobID               id.JobID
}

func NewEdgeExecution(
	id EdgeExecutionID,
	edgeID string,
	jobID id.JobID,
	intermediateDataURL *string,
) *EdgeExecution {
	return &EdgeExecution{
		id:                  id,
		edgeID:              edgeID,
		jobID:               jobID,
		intermediateDataURL: intermediateDataURL,
	}
}

func (e *EdgeExecution) ID() EdgeExecutionID {
	return e.id
}

func (e *EdgeExecution) EdgeID() string {
	return e.edgeID
}

func (e *EdgeExecution) JobID() id.JobID {
	return e.jobID
}

func (e *EdgeExecution) IntermediateDataURL() *string {
	return e.intermediateDataURL
}
