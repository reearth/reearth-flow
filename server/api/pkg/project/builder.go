package project

import (
	"time"
)

type Builder struct {
	p *Project
}

func New() *Builder {
	return &Builder{p: &Project{}}
}

func (b *Builder) BasicAuthPassword(basicAuthPassword string) *Builder {
	b.p.basicAuthPassword = basicAuthPassword
	return b
}

func (b *Builder) BasicAuthUsername(basicAuthUsername string) *Builder {
	b.p.basicAuthUsername = basicAuthUsername
	return b
}

func (b *Builder) Build() (*Project, error) {
	if b.p.id.IsNil() {
		return nil, ErrInvalidID
	}

	if b.p.updatedAt.IsZero() {
		b.p.updatedAt = b.p.CreatedAt()
	}
	return b.p, nil
}

func (b *Builder) Description(description string) *Builder {
	b.p.description = description
	return b
}

func (b *Builder) ID(id ID) *Builder {
	b.p.id = id
	return b
}

func (b *Builder) IsArchived(isArchived bool) *Builder {
	b.p.isArchived = isArchived
	return b
}

func (b *Builder) IsBasicAuthActive(isBasicAuthActive bool) *Builder {
	b.p.isBasicAuthActive = isBasicAuthActive
	return b
}

func (b *Builder) MustBuild() *Project {
	r, err := b.Build()
	if err != nil {
		panic(err)
	}
	return r
}

func (b *Builder) Name(name string) *Builder {
	b.p.name = name
	return b
}

func (b *Builder) NewID() *Builder {
	b.p.id = NewID()
	return b
}

func (b *Builder) SharedToken(sharedToken *string) *Builder {
	b.p.sharedToken = sharedToken
	return b
}

func (b *Builder) UpdatedAt(updatedAt time.Time) *Builder {
	b.p.updatedAt = updatedAt
	return b
}

func (b *Builder) Workflow(workflow WorkflowID) *Builder {
	b.p.workflow = workflow
	return b
}

func (b *Builder) Workspace(workspace WorkspaceID) *Builder {
	b.p.workspace = workspace
	return b
}
