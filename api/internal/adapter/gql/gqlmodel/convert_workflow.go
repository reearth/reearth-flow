package gqlmodel

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/idx"
)

func FromInputWorkflow(pid id.ProjectID, wsid idx.ID[accountdomain.Workspace], w []*InputWorkflow) *workflow.Workflow {
	if w == nil {
		return nil
	}

	wfid := workflow.NewWorkflowID()

	entryGraphID, graphs := convertWorkflows(w)

	yaml, err := workflow.ToWorkflowYaml(wfid, "Debug workflow", entryGraphID, nil, graphs)
	if err != nil {
		return nil
	}

	return workflow.NewWorkflow(wfid, pid, wsid, yaml)
}

func convertWorkflows(w []*InputWorkflow) (string, []*workflow.Graph) {
	var entryGraphID string
	graphs := []*workflow.Graph{}

	for _, v := range w {
		if *v.IsMain {
			eGraphID, err := ToID[id.Graph](v.ID)
			if err == nil {
				entryGraphID = eGraphID.String()
			}
		}

		nodes := convertNodes(v.Nodes)
		edges := convertEdges(v.Edges)
		graphs = append(graphs, workflow.NewGraph(workflow.NewGraphID(), v.Name, nodes, edges))
	}

	return entryGraphID, graphs
}

func convertNodes(inputNodes []*InputWorkflowNode) []*workflow.Node {
	nodes := []*workflow.Node{}
	for _, n := range inputNodes {
		nID, _ := ToID[id.Node](n.ID)
		newNode := workflow.NewNode(nID, n.Data.Name, n.Type.String(), string(n.Data.ActionID), nil) // TODO: Need to add params here
		nodes = append(nodes, newNode)
	}
	return nodes
}

func convertEdges(inputEdges []*InputWorkflowEdge) []*workflow.Edge {
	edges := []*workflow.Edge{}
	for _, e := range inputEdges {
		eID, _ := ToID[id.Edge](e.ID)
		newEdge := workflow.NewEdge(eID, string(e.Source), string(e.Target), e.SourceHandle, e.TargetHandle)
		edges = append(edges, newEdge)
	}
	return edges
}
