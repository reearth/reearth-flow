package gqlmodel

import (
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
	"github.com/stretchr/testify/assert"
)

func TestConvertWorkflows(t *testing.T) {
	workflowID := workflow.NewID()
	nodeID1 := workflow.NewNodeID()
	edgeID1 := workflow.NewEdgeID()
	input := []*InputWorkflow{
		{
			ID:     IDFrom(workflowID),
			Name:   "Test Workflow",
			IsMain: ptrBool(true),
			Nodes: []*InputWorkflowNode{
				{
					ID:   IDFrom(nodeID1),
					Type: "READER",
					Data: &InputData{
						Name:     "Node 1",
						ActionID: "action1",
					},
				},
			},
			Edges: []*InputWorkflowEdge{
				{
					ID:           IDFrom(edgeID1),
					Source:       "node1",
					Target:       "node2",
					SourceHandle: "handle1",
					TargetHandle: "handle2",
				},
			},
		},
	}

	graph1ID, _ := id.GraphIDFrom(workflowID.String())
	expectedGraphs := []*workflow.Graph{
		workflow.NewGraph(graph1ID, "Test Workflow", []*workflow.Node{
			workflow.NewNode(nodeID1, "Node 1", "READER", "action1", nil),
		}, []*workflow.Edge{
			workflow.NewEdge(edgeID1, "node1", "node2", "handle1", "handle2"),
		}),
	}

	entryGraphID, graphs := convertWorkflows(input)
	assert.Equal(t, workflowID.String(), entryGraphID)
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
