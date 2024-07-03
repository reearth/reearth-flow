package workflow

import "testing"

func TestWorkflow_SetID(t *testing.T) {
	w := &Workflow{}
	wId := NewID()
	w.SetID(wId)
	if w.id != wId {
		t.Errorf("expected %s, got %s", wId, w.id)
	}
}

func TestWorkflow_SetName(t *testing.T) {
	w := &Workflow{}
	wName := "name"
	w.SetName(wName)
	if w.name != wName {
		t.Errorf("expected %s, got %s", wName, w.name)
	}
}

func TestWorkflow_SetEntryGraphId(t *testing.T) {
	w := &Workflow{}
	wEntryGraphId := "entryGraphId"
	w.SetEntryGraphId(wEntryGraphId)
	if w.entryGraphId != wEntryGraphId {
		t.Errorf("expected %s, got %s", wEntryGraphId, w.entryGraphId)
	}
}

func TestWorkflow_SetWith(t *testing.T) {
	w := &Workflow{}
	wWith := map[string]interface{}{"key": "value"}
	w.SetWith(wWith)
	if w.with["key"] != "value" {
		t.Errorf("expected %v, got %v", wWith, w.with)
	}
}

func TestWorkflow_SetGraphs(t *testing.T) {
	w := &Workflow{}
	wGraphs := []Graph{{}}
	w.SetGraphs(wGraphs)
	if len(w.graphs) != 1 {
		t.Errorf("expected %v, got %v", wGraphs, w.graphs)
	}
}
