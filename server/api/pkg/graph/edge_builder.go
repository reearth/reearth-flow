package graph

import "github.com/reearth/reearth-flow/api/pkg/id"

type EdgeExecutionBuilder struct {
	e *EdgeExecution
}

func NewEdgeExecutionBuilder() *EdgeExecutionBuilder {
	return &EdgeExecutionBuilder{e: &EdgeExecution{}}
}

func (b *EdgeExecutionBuilder) Build() (*EdgeExecution, error) {
	if b.e.id.IsNil() {
		return nil, ErrInvalidID
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

func (b *EdgeExecutionBuilder) ID(id EdgeExecutionID) *EdgeExecutionBuilder {
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

func (b *EdgeExecutionBuilder) IntermediateDataURL(url *string) *EdgeExecutionBuilder {
	b.e.intermediateDataURL = url
	return b
}
