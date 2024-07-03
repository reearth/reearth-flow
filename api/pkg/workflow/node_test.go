package workflow

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestNode_SetID(t *testing.T) {
	n := &Node{}
	n.SetID("id")
	assert.Equal(t, "id", n.id)
}

func TestNode_SetName(t *testing.T) {
	n := &Node{}
	n.SetName("name")
	assert.Equal(t, "name", n.name)
}

func TestNode_SetNodeType(t *testing.T) {
	n := &Node{}
	n.SetNodeType("type")
	assert.Equal(t, "type", n.nodeType)
}

func TestNode_SetAction(t *testing.T) {
	n := &Node{}
	n.SetAction("action")
	assert.Equal(t, "action", n.action)
}

func TestNode_SetWith(t *testing.T) {
	n := &Node{}
	n.SetWith(map[string]interface{}{"key": "value"})
	assert.Equal(t, map[string]interface{}{"key": "value"}, n.with)
}
