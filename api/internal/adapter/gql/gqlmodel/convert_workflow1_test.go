package gqlmodel

import (
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
	"github.com/stretchr/testify/assert"
)

// func TestFromInputWorkflow(t *testing.T) {
// 	pid := project.NewID()
// 	wsid := accountdomain.NewWorkspaceID()

// 	nodeID1 := ID("node1")
// 	edgeID1 := ID("edge1")

// 	input := []*InputWorkflow{
// 		{
// 			ID:     "workflow1",
// 			Name:   "Test Workflow",
// 			IsMain: ptrBool(true),
// 			Nodes: []*InputWorkflowNode{
// 				{
// 					ID:   nodeID1,
// 					Type: "READER",
// 					Data: &InputData{
// 						Name:     "Node 1",
// 						ActionID: "action1",
// 					},
// 				},
// 			},
// 			Edges: []*InputWorkflowEdge{
// 				{
// 					ID:           edgeID1,
// 					Source:       "node1",
// 					Target:       "node2",
// 					SourceHandle: "handle1",
// 					TargetHandle: "handle2",
// 				},
// 			},
// 		},
// 	}

// 	expectedEntryGraphID := "workflow1"
// 	expectedNodeID1, _ := ToID[id.Node](nodeID1)
// 	expectedEdgeID1, _ := ToID[id.Edge](edgeID1)

// 	expectedGraphs := []*workflow.Graph{
// 		workflow.NewGraph(workflow.NewGraphID(), "Test Workflow", []*workflow.Node{
// 			workflow.NewNode(expectedNodeID1, "Node 1", "READER", "action1", nil),
// 		}, []*workflow.Edge{
// 			workflow.NewEdge(expectedEdgeID1, "node1", "node2", "handle1", "handle2"),
// 		}),
// 	}

// 	result := FromInputWorkflow(pid, wsid, input)
// 	assert.NotNil(t, result)
// 	assert.Equal(t, expectedEntryGraphID, result.EntryGraphID())
// 	assert.Equal(t, expectedGraphs, result.Graphs())
// }

func TestConvertWorkflows(t *testing.T) {
	nodeID1 := ID("node1")
	edgeID1 := ID("edge1")

	input := []*InputWorkflow{
		{
			ID:     "workflow1",
			Name:   "Test Workflow",
			IsMain: ptrBool(true),
			Nodes: []*InputWorkflowNode{
				{
					ID:   "node1",
					Type: "READER",
					Data: &InputData{
						Name:     "Node 1",
						ActionID: "action1",
					},
				},
			},
			Edges: []*InputWorkflowEdge{
				{
					ID:           "edge1",
					Source:       "node1",
					Target:       "node2",
					SourceHandle: "handle1",
					TargetHandle: "handle2",
				},
			},
		},
	}

	expectedEntryGraphID := "workflow1"
	expectedNodeID1, _ := ToID[id.Node](nodeID1)
	expectedEdgeID1, _ := ToID[id.Edge](edgeID1)

	expectedGraphs := []*workflow.Graph{
		workflow.NewGraph(workflow.NewGraphID(), "Test Workflow", []*workflow.Node{
			workflow.NewNode(expectedNodeID1, "Node 1", "READER", "action1", nil),
		}, []*workflow.Edge{
			workflow.NewEdge(expectedEdgeID1, "node1", "node2", "handle1", "handle2"),
		}),
	}

	entryGraphID, graphs := convertWorkflows(input)
	assert.Equal(t, expectedEntryGraphID, entryGraphID)
	assert.Equal(t, expectedGraphs, graphs)
}

func TestConvertNodes(t *testing.T) {
	nodeID1 := ID("node1")

	input := []*InputWorkflowNode{
		{
			ID:   "node1",
			Type: "READER",
			Data: &InputData{
				Name:     "Node 1",
				ActionID: "action1",
			},
		},
	}

	expectedNodeID1, _ := ToID[id.Node](nodeID1)
	expected := []*workflow.Node{
		workflow.NewNode(expectedNodeID1, "Node 1", "READER", "action1", nil),
	}

	result := convertNodes(input)
	assert.Equal(t, expected, result)
}

func TestConvertEdges(t *testing.T) {
	edgeID1 := ID("edge1")

	input := []*InputWorkflowEdge{
		{
			ID:           "edge1",
			Source:       "node1",
			Target:       "node2",
			SourceHandle: "handle1",
			TargetHandle: "handle2",
		},
	}

	expectedEdgeID1, _ := ToID[id.Edge](edgeID1)
	expected := []*workflow.Edge{
		workflow.NewEdge(expectedEdgeID1, "node1", "node2", "handle1", "handle2"),
	}

	result := convertEdges(input)
	assert.Equal(t, expected, result)
}

func ptrBool(b bool) *bool {
	return &b
}
