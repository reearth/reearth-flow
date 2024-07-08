package workflow

import (
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
)

func TestNewWorkflow(t *testing.T) {
	workflowID := id.NewWorkflowID()
	name := "Test Workflow"
	entryGraphId := "graph1"
	with := map[string]interface{}{"key": "value"}

	nodeID1 := id.NewNodeID()
	nodeID2 := id.NewNodeID()

	edgeID1 := id.NewEdgeID()

	graphs := []Graph{
		{
			id:   id.NewGraphID(),
			name: "Test Graph",
			nodes: []Node{
				{
					id:       nodeID1,
					name:     "Test Node",
					nodeType: "Test Type",
					action:   "Test Action",
					with:     map[string]interface{}{"key1": "value1"},
				},
				{
					id:       nodeID2,
					name:     "Test Node 2",
					nodeType: "Test Type 2",
					action:   "Test Action 2",
					with:     map[string]interface{}{"key2": "value2"},
				},
			},
			edges: []Edge{
				{
					id:       edgeID1,
					from:     nodeID1.String(),
					to:       nodeID2.String(),
					toPort:   "to-port",
					fromPort: "from-port",
				},
			},
		},
	}

	result := NewWorkflow(workflowID, name, entryGraphId, with, graphs)

	want := &Workflow{
		id:           workflowID,
		name:         name,
		entryGraphId: entryGraphId,
		with:         with,
		graphs:       graphs,
	}

	assert.Equal(t, result, want)
}
