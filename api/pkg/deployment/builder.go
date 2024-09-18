package deployment

import (
	"time"
)

type DeploymentBuilder struct {
	d *Deployment
}

func New() *DeploymentBuilder {
	return &DeploymentBuilder{d: &Deployment{}}
}

func (b *DeploymentBuilder) Build() (*Deployment, error) {
	if b.d.id.IsNil() {
		return nil, ErrInvalidID
	}
	return b.d, nil
}

func (b *DeploymentBuilder) MustBuild() *Deployment {
	r, err := b.Build()
	if err != nil {
		panic(err)
	}
	return r
}

func (b *DeploymentBuilder) ID(id ID) *DeploymentBuilder {
	b.d.id = id
	return b
}

func (b *DeploymentBuilder) NewID() *DeploymentBuilder {
	b.d.id = NewID()
	return b
}

func (b *DeploymentBuilder) Project(project ProjectID) *DeploymentBuilder {
	b.d.project = project
	return b
}

func (b *DeploymentBuilder) Workspace(workspace WorkspaceID) *DeploymentBuilder {
	b.d.workspace = workspace
	return b
}

func (b *DeploymentBuilder) Workflow(workflow WorkflowID) *DeploymentBuilder {
	b.d.workflow = workflow
	return b
}

func (b *DeploymentBuilder) Version(version string) *DeploymentBuilder {
	b.d.version = version
	return b
}

func (b *DeploymentBuilder) CreatedAt(createdAt time.Time) *DeploymentBuilder {
	b.d.createdAt = createdAt
	return b
}

func (b *DeploymentBuilder) UpdatedAt(updatedAt time.Time) *DeploymentBuilder {
	b.d.updatedAt = updatedAt
	return b
}
