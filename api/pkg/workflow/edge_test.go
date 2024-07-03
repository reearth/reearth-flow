package workflow

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestEdge_SetID(t *testing.T) {
	e := &Edge{}
	e.SetID("id")
	assert.Equal(t, "id", e.id)
}

func TestEdge_SetFrom(t *testing.T) {
	e := &Edge{}
	e.SetFrom([]string{"from"})
	assert.Equal(t, []string{"from"}, e.from)
}

func TestEdge_SetTo(t *testing.T) {
	e := &Edge{}
	e.SetTo([]string{"to"})
	assert.Equal(t, []string{"to"}, e.to)
}

func TestEdge_SetFromPort(t *testing.T) {
	e := &Edge{}
	e.SetFromPort("fromPort")
	assert.Equal(t, "fromPort", e.fromPort)
}

func TestEdge_SetToPort(t *testing.T) {
	e := &Edge{}
	e.SetToPort("toPort")
	assert.Equal(t, "toPort", e.toPort)
}
