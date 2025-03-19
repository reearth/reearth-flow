package edge

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
)

type Status string

const (
	StatusInProgress Status = "IN_PROGRESS"
	StatusCompleted  Status = "COMPLETED"
	StatusFailed     Status = "FAILED"
)

type EdgeExecution struct {
	id                  string
	edgeID              string
	jobID               id.JobID
	startedAt           *time.Time
	completedAt         *time.Time
	status              Status
	featureID           *string
	intermediateDataURL *string
}

func NewEdgeExecution(
	id string,
	edgeID string,
	jobID id.JobID,
	workflowID string,
	status Status,
	startedAt *time.Time,
	completedAt *time.Time,
	featureID *string,
	intermediateDataURL *string,
) *EdgeExecution {
	return &EdgeExecution{
		id:                  id,
		edgeID:              edgeID,
		jobID:               jobID,
		status:              status,
		startedAt:           startedAt,
		completedAt:         completedAt,
		featureID:           featureID,
		intermediateDataURL: intermediateDataURL,
	}
}

func (e *EdgeExecution) ID() string {
	return e.id
}

func (e *EdgeExecution) EdgeID() string {
	return e.edgeID
}

func (e *EdgeExecution) JobID() id.JobID {
	return e.jobID
}

func (e *EdgeExecution) Status() Status {
	return e.status
}

func (e *EdgeExecution) StartedAt() *time.Time {
	return e.startedAt
}

func (e *EdgeExecution) CompletedAt() *time.Time {
	return e.completedAt
}

func (e *EdgeExecution) FeatureID() *string {
	return e.featureID
}

func (e *EdgeExecution) IntermediateDataURL() *string {
	return e.intermediateDataURL
}
