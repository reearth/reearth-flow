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
	if b.t.eventSource == "" {
		return nil, errors.New("event source is required")
	}
	if b.t.createdAt.IsZero() {
		b.t.createdAt = time.Now()
	}
	if b.t.updatedAt.IsZero() {
		b.t.updatedAt = time.Now()
	}
	return b.t, nil
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
