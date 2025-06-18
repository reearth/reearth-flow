package parameter

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
)

type Builder struct {
	p Parameter
}

func New() *Builder {
	return &Builder{}
}

func (b *Builder) Build() (*Parameter, error) {
	if b.p.id.IsNil() {
		b.p.id = id.NewParameterID()
	}
	if b.p.createdAt.IsZero() {
		b.p.createdAt = time.Now()
	}
	if b.p.updatedAt.IsZero() {
		b.p.updatedAt = time.Now()
	}
	return &b.p, nil
}

func (b *Builder) CreatedAt(t time.Time) *Builder {
	b.p.createdAt = t
	return b
}

func (b *Builder) ID(id ID) *Builder {
	b.p.id = id
	return b
}

func (b *Builder) Index(index int) *Builder {
	b.p.index = index
	return b
}

func (b *Builder) Name(name string) *Builder {
	b.p.name = name
	return b
}

func (b *Builder) ProjectID(id ProjectID) *Builder {
	b.p.projectID = id
	return b
}

func (b *Builder) Required(required bool) *Builder {
	b.p.required = required
	return b
}

func (b *Builder) Type(t Type) *Builder {
	b.p.typ = t
	return b
}

func (b *Builder) UpdatedAt(t time.Time) *Builder {
	b.p.updatedAt = t
	return b
}

func (b *Builder) DefaultValue(defaultValue any) *Builder {
	b.p.defaultValue = defaultValue
	return b
}

func (b *Builder) Public(public bool) *Builder {
	b.p.public = public
	return b
}
