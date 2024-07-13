package workflow

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestEdgeSetters(t *testing.T) {
	e := &Edge{}
	edgeID := NewEdgeID()
	e.SetID(edgeID)
	assert.Equal(t, edgeID, e.id, "SetID should correctly set the id field")
	fromNodes := "node1"
	e.SetFrom(fromNodes)
	assert.Equal(t, fromNodes, e.from, "SetFrom should correctly set the from field")
	toNodes := "node2"
	e.SetTo(toNodes)
	assert.Equal(t, toNodes, e.to, "SetTo should correctly set the to field")
	e.SetFromPort("fromPort")
	assert.Equal(t, "fromPort", e.fromPort, "SetFromPort should correctly set the fromPort field")
	e.SetToPort("toPort")
	assert.Equal(t, "toPort", e.toPort, "SetToPort should correctly set the toPort field")
}
