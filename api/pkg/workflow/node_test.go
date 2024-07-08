package workflow

import (
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
)

func TestNodeSetters(t *testing.T) {
	n := &Node{}
	nodeID := id.NewNodeID()
	n.SetID(nodeID)
	assert.Equal(t, nodeID, n.id, "SetID should correctly set the id field")
	n.SetName("testName")
	assert.Equal(t, "testName", n.name, "SetName should correctly set the name field")
	n.SetNodeType("testType")
	assert.Equal(t, "testType", n.nodeType, "SetNodeType should correctly set the nodeType field")
	n.SetAction("testAction")
	assert.Equal(t, "testAction", n.action, "SetAction should correctly set the action field")
	with := map[string]interface{}{"key": "value"}
	n.SetWith(with)
	assert.Equal(t, with, n.with, "SetWith should correctly set the with field")

}
