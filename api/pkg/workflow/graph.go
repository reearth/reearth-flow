package workflow

import "github.com/reearth/reearth-flow/api/pkg/id"

type Graph struct {
	id    id.GraphID
	name  string
	nodes []*Node
	edges []*Edge
}

func NewGraph(id id.GraphID, name string, nodes []*Node, edges []*Edge) *Graph {
	return &Graph{
		id:    id,
		name:  name,
		nodes: nodes,
		edges: edges,
	}
}

func (g *Graph) ID() id.GraphID {
	return g.id
}

func (g *Graph) Name() string {
	return g.name
}

func (g *Graph) Nodes() []*Node {
	return g.nodes
}

func (g *Graph) Edges() []*Edge {
	return g.edges
}
