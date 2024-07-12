package workflow

import "github.com/reearth/reearth-flow/api/pkg/id"

type Graph struct {
	ID    id.GraphID
	Name  string
	Nodes []*Node
	Edges []*Edge
}

func NewGraph(id id.GraphID, name string, nodes []*Node, edges []*Edge) *Graph {
	return &Graph{
		ID:    id,
		Name:  name,
		Nodes: nodes,
		Edges: edges,
	}
}
