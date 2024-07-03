package workflow

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestEdge_ID(t *testing.T) {
	e := &Edges{id: "id"}
	assert.Equal(t, "id", e.ID())
}

func TestEdge_SetSource(t *testing.T) {
	e := &Edges{}
	e.SetSource([]string{"source"})
	assert.Equal(t, []string{"source"}, e.source)
}

func TestEdge_SetSourceHandle(t *testing.T) {
	e := &Edges{}
	e.SetSourceHandle("sourceHandle")
	assert.Equal(t, "sourceHandle", e.sourceHandle)
}

func TestEdge_SetTarget(t *testing.T) {
	e := &Edges{}
	e.SetTarget([]string{"target"})
	assert.Equal(t, []string{"target"}, e.target)
}

func TestEdge_SetTargetHandle(t *testing.T) {
	e := &Edges{}
	e.SetTargetHandle("targetHandle")
	assert.Equal(t, "targetHandle", e.targetHandle)
}
