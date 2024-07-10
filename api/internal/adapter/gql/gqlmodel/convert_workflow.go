package gqlmodel

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
)

func FromInputWorkflow(pid id.ProjectID, w []*InputWorkflow) *workflow.Workflow {
	if w == nil {
		return nil
	}

	var entryGraphID string

	graphs := []*workflow.Graph{}

	for _, v := range w {
		if *v.IsMain {
			eGraphID, err := ToID[id.Graph](v.ID)
			if err != nil {
				entryGraphID = eGraphID.String()
			}
		}

		nodes := []*workflow.Node{}
		for _, n := range v.Nodes {
			nID, _ := ToID[id.Node](n.ID)
			nodes = append(nodes, workflow.NewNode(nID, n.Data.Name, n.Type.String(), string(n.Data.ActionID), nil)) // TODO: Need to add params here
		}
		edges := []*workflow.Edge{}
		for _, e := range v.Edges {
			eID, _ := ToID[id.Edge](e.ID)
			edges = append(edges, workflow.NewEdge(eID, string(e.Source), string(e.Target), e.SourceHandle, e.TargetHandle))
		}

		graphs = append(graphs, workflow.NewGraph(workflow.NewGraphID(), v.Name, nodes, edges))
	}

	return workflow.NewWorkflow(workflow.NewWorkflowID(), workflow.NewWorkspaceID(), pid, "asdfsdf", entryGraphID, nil, graphs)
}
