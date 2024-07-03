package workflow

import "testing"

func TestGraph_SetID(t *testing.T) {
	g := &Graph{}
	gId := NewID()
	g.SetID(gId)
	if g.id != gId {
		t.Errorf("expected %s, got %s", gId, g.id)
	}
}

func TestGraph_SetName(t *testing.T) {
	g := &Graph{}
	gName := "name"
	g.SetName(gName)
	if g.name != gName {
		t.Errorf("expected %s, got %s", gName, g.name)
	}
}

func TestGraph_SetNodes(t *testing.T) {
	g := &Graph{}
	gNodes := []Node{{}}
	g.SetNodes(gNodes)
	if len(g.nodes) != 1 {
		t.Errorf("expected %v, got %v", gNodes, g.nodes)
	}
}

func TestGraph_SetEdges(t *testing.T) {
	g := &Graph{}
	gEdges := []Edge{{}}
	g.SetEdges(gEdges)
	if len(g.edges) != 1 {
		t.Errorf("expected %v, got %v", gEdges, g.edges)
	}
}
