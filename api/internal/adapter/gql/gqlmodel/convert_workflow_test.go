package gqlmodel

import (
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
	"github.com/stretchr/testify/assert"
)

func TestConvertGraphs(t *testing.T) {
	graphID1 := ID("graph1")
	nodeType := "READER"
	actionID := "action1"
	nodeID1 := ID("node1")
	edgeID1 := ID("edge1")

	input := []*InputGraph{
		{
			ID:   graphID1,
			Name: "Graph 1",
			Nodes: []*InputWorkflowNode{
				{
					ID:         nodeID1,
					Type:       &nodeType,
					Name:       "Node 1",
					Action:     &actionID,
					SubGraphID: nil,
					With:       nil,
				},
			},
			Edges: []*InputWorkflowEdge{
				{
					ID:       edgeID1,
					From:     "node1",
					To:       "node2",
					FromPort: "handle1",
					ToPort:   "handle2",
				},
			},
		},
	}

	expectedGraphID1, _ := ToID[id.Graph](graphID1)
	expectedNodeID1, _ := ToID[id.Node](nodeID1)
	expectedEdgeID1, _ := ToID[id.Edge](edgeID1)
	expected := []*workflow.Graph{
		workflow.NewGraph(expectedGraphID1, "Graph 1", []*workflow.Node{
			workflow.NewNode(expectedNodeID1, "Node 1", "READER", "action1", nil),
		}, []*workflow.Edge{
			workflow.NewEdge(expectedEdgeID1, "node1", "node2", "handle1", "handle2"),
		}),
	}

	result := convertGraphs(input)

	assert.Equal(t, expected, result)
}

func TestConvertNodes(t *testing.T) {
	nodeID1 := ID("node1")
	nodeType := "READER"
	actionID := "action1"

	input := []*InputWorkflowNode{
		{
			ID:         "node1",
			Type:       &nodeType,
			Name:       "Node 1",
			Action:     &actionID,
			SubGraphID: nil,
			With:       nil,
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
			ID:       "edge1",
			From:     "node1",
			To:       "node2",
			FromPort: "handle1",
			ToPort:   "handle2",
		},
	}

	expectedEdgeID1, _ := ToID[id.Edge](edgeID1)
	expected := []*workflow.Edge{
		workflow.NewEdge(expectedEdgeID1, "node1", "node2", "handle1", "handle2"),
	}

	result := convertEdges(input)
	assert.Equal(t, expected, result)
}
