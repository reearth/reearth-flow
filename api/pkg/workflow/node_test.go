package workflow

import (
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
)

func TestNewNode(t *testing.T) {
	nodeID := id.NewNodeID()
	name := "name"
	nodeType := "type"
	action := "action"
	with := map[string]interface{}{"key": "value"}

	result := NewNode(nodeID, name, nodeType, action, with)

	want := &Node{
		ID:       nodeID,
		Name:     name,
		NodeType: nodeType,
		Action:   action,
		With:     with,
	}

	assert.Equal(t, result, want)
}
