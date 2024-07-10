package workflow

import (
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
)

func TestNewGraph(t *testing.T) {
	graphID := id.NewGraphID()
	name := "name"
	nodes := []*Node{}
	edges := []*Edge{}

	result := NewGraph(graphID, name, nodes, edges)

	want := &Graph{
		id:    graphID,
		name:  name,
		nodes: nodes,
		edges: edges,
	}

	assert.Equal(t, result, want)
}
