package workflow

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestNew(t *testing.T) {
	b := New()
	assert.NotNil(t, b)
	assert.NotNil(t, b.w)
}

func TestBuilder_Build(t *testing.T) {
	b := New()
	_, err := b.Build()
	assert.Error(t, err)
}

func TestBuilder_ID(t *testing.T) {
	b := New()
	id := NewID()
	b.ID(id)
	assert.Equal(t, id, b.w.id)
}

func TestBuilder_Name(t *testing.T) {
	b := New()
	name := "name"
	b.Name(name)
	assert.Equal(t, name, b.w.name)
}

func TestBuilder_EntryGraphID(t *testing.T) {
	b := New()
	id := "id"
	b.EntryGraphID(id)
	assert.Equal(t, id, b.w.entryGraphId)
}

func TestBuilder_With(t *testing.T) {
	b := New()
	with := map[string]interface{}{"a": 1}
	b.With(with)
	assert.Equal(t, with, b.w.with)
}

func TestBuilder_Graphs(t *testing.T) {
	b := New()
	graphs := []Graph{{}}
	b.Graphs(graphs)
	assert.Equal(t, graphs, b.w.graphs)
}
