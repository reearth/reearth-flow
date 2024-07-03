package workflow

type Graph struct {
	id    ID
	name  string
	nodes []Node
	edges []Edge
}

func (g *Graph) ID() ID {
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

func (g *Graph) SetID(id ID) {
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
