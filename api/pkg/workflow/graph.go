package workflow

import "github.com/reearth/reearth-flow/api/pkg/id"

type Graph struct {
	id    id.GraphID
	name  string
	nodes []Node
	edges []Edge
}

func (g *Graph) ID() id.GraphID {
	return g.id
}

func (g *Graph) Name() string {
	return g.name
}

func (g *Graph) Nodes() []Node {
	return g.nodes
}

func (g *Graph) Edges() []Edge {
	return g.edges
}

func (g *Graph) SetID(id id.GraphID) {
	g.id = id
}

func (g *Graph) SetName(name string) {
	g.name = name
}

func (g *Graph) SetNodes(nodes []Node) {
	g.nodes = nodes
}

func (g *Graph) SetEdges(edges []Edge) {
	g.edges = edges
}
