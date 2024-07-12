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

	var entryGraphID string

	graphs := []*workflow.Graph{}

	for _, v := range w {
		if *v.IsMain {
			eGraphID, err := ToID[id.Graph](v.ID)
			if err == nil {
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

	yaml := workflow.ToWorkflowYaml(wfid, "Debug workflow", entryGraphID, nil, graphs)

	return workflow.NewWorkflow(wfid, pid, wsid, yaml)
}
