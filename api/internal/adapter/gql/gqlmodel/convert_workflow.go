package gqlmodel

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/idx"
)

func FromInputWorkflow(pid id.ProjectID, wsid idx.ID[accountdomain.Workspace], w *InputWorkflow) *workflow.Workflow {
	if w == nil {
		return nil
	}

	wfid := workflow.NewWorkflowID()

	graphs := convertGraphs(w.Graphs)

	yaml, err := workflow.ToWorkflowYaml(wfid, "Debug workflow", string(w.EntryGraphID), nil, graphs)
	if err != nil {
		return nil
	}

	return workflow.NewWorkflow(wfid, pid, wsid, yaml)
}

func convertGraphs(w []*InputGraph) []*workflow.Graph {
	graphs := []*workflow.Graph{}

	for _, v := range w {
		graphID, _ := ToID[id.Graph](v.ID)
		nodes := convertNodes(v.Nodes)
		edges := convertEdges(v.Edges)
		graphs = append(graphs, workflow.NewGraph(graphID, v.Name, nodes, edges))
	}

	return graphs
}

func convertNodes(inputNodes []*InputWorkflowNode) []*workflow.Node {
	nodes := []*workflow.Node{}
	for _, n := range inputNodes {
		nID, _ := ToID[id.Node](n.ID)
		newNode := workflow.NewNode(nID, n.Name, *n.Type, *n.Action, nil) // TODO: Need to add params here
		nodes = append(nodes, newNode)
	}
	return nodes
}

func convertEdges(inputEdges []*InputWorkflowEdge) []*workflow.Edge {
	edges := []*workflow.Edge{}
	for _, e := range inputEdges {
		eID, _ := ToID[id.Edge](e.ID)
		newEdge := workflow.NewEdge(eID, string(e.From), string(e.To), e.FromPort, e.ToPort)
		edges = append(edges, newEdge)
	}
	return edges
}
