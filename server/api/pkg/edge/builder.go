package edge

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
)

type EdgeExecutionBuilder struct {
	e *EdgeExecution
}

func New() *EdgeExecutionBuilder {
	return &EdgeExecutionBuilder{e: &EdgeExecution{}}
}

func (b *EdgeExecutionBuilder) Build() (*EdgeExecution, error) {
	if b.e.id == "" {
		return nil, id.ErrInvalidID
	}
	return b.e, nil
}

func (b *EdgeExecutionBuilder) MustBuild() *EdgeExecution {
	r, err := b.Build()
	if err != nil {
		panic(err)
	}
	return r
}

func (b *EdgeExecutionBuilder) ID(id string) *EdgeExecutionBuilder {
	b.e.id = id
	return b
}

func (b *EdgeExecutionBuilder) EdgeID(edgeID string) *EdgeExecutionBuilder {
	b.e.edgeID = edgeID
	return b
}

func (b *EdgeExecutionBuilder) JobID(jobID id.JobID) *EdgeExecutionBuilder {
	b.e.jobID = jobID
	return b
}

func (b *EdgeExecutionBuilder) WorkflowID(workflowID string) *EdgeExecutionBuilder {
	b.e.workflowID = workflowID
	return b
}

func (b *EdgeExecutionBuilder) Status(status Status) *EdgeExecutionBuilder {
	b.e.status = status
	return b
}

func (b *EdgeExecutionBuilder) StartedAt(startedAt *time.Time) *EdgeExecutionBuilder {
	b.e.startedAt = startedAt
	return b
}

func (b *EdgeExecutionBuilder) CompletedAt(completedAt *time.Time) *EdgeExecutionBuilder {
	b.e.completedAt = completedAt
	return b
}

func (b *EdgeExecutionBuilder) FeatureID(featureID *string) *EdgeExecutionBuilder {
	b.e.featureID = featureID
	return b
}

func (b *EdgeExecutionBuilder) IntermediateDataURL(url *string) *EdgeExecutionBuilder {
	b.e.intermediateDataURL = url
	return b
}
