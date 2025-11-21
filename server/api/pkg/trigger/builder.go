package trigger

import (
	"errors"
	"time"
)

type Builder struct {
	t *Trigger
}

func New() *Builder {
	return &Builder{t: &Trigger{}}
}

func (b *Builder) Build() (*Trigger, error) {
	if b.t.id.IsNil() {
		return nil, errors.New("id is required")
	}
	if b.t.workspaceId.IsNil() {
		return nil, errors.New("workspace is required")
	}
	if b.t.deploymentId.IsNil() {
		return nil, errors.New("deployment is required")
	}
	if b.t.description == "" {
		return nil, errors.New("description is required")
	}
	if b.t.eventSource == "" {
		return nil, errors.New("event source is required")
	}
	if b.t.createdAt.IsZero() {
		b.t.createdAt = time.Now()
	}
	if b.t.updatedAt.IsZero() {
		b.t.updatedAt = b.t.CreatedAt()
	}
	return b.t, nil
}

func (b *Builder) MustBuild() *Trigger {
	r, err := b.Build()
	if err != nil {
		panic(err)
	}
	return r
}

func (b *Builder) ID(id ID) *Builder {
	b.t.id = id
	return b
}

func (b *Builder) NewID() *Builder {
	b.t.id = NewID()
	return b
}

func (b *Builder) Workspace(workspace WorkspaceID) *Builder {
	b.t.workspaceId = workspace
	return b
}

func (b *Builder) Deployment(deployment DeploymentID) *Builder {
	b.t.deploymentId = deployment
	return b
}

func (b *Builder) Description(description string) *Builder {
	b.t.description = description
	return b
}

func (b *Builder) EventSource(eventSource EventSourceType) *Builder {
	b.t.eventSource = eventSource
	return b
}

func (b *Builder) AuthToken(token string) *Builder {
	b.t.authToken = &token
	return b
}

func (b *Builder) TimeInterval(interval TimeInterval) *Builder {
	b.t.timeInterval = &interval
	return b
}

func (b *Builder) Variables(variables map[string]string) *Builder {
	b.t.variables = variables
	return b
}

func (b *Builder) CreatedAt(createdAt time.Time) *Builder {
	b.t.createdAt = createdAt
	return b
}

func (b *Builder) UpdatedAt(updatedAt time.Time) *Builder {
	b.t.updatedAt = updatedAt
	return b
}

func (b *Builder) LastTriggered(lastTriggered time.Time) *Builder {
	b.t.lastTriggered = &lastTriggered
	return b
}
