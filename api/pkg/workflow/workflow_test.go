package workflow

import "testing"

func TestWorkflowSetters(t *testing.T) {
	w := &Workflow{}
	wId := NewID()
	w.SetID(wId)
	if w.id != wId {
		t.Errorf("expected %s, got %s", wId, w.id)
	}
	w.SetName("testName")
	if w.name != "testName" {
		t.Errorf("expected %s, got %s", "testName", w.name)
	}
	w.SetEntryGraphId("testEntryGraphId")
	if w.entryGraphId != "testEntryGraphId" {
		t.Errorf("expected %s, got %s", "testEntryGraphId", w.entryGraphId)
	}
	wWith := map[string]interface{}{"key": "value"}
	w.SetWith(wWith)
	if w.with["key"] != "value" {
		t.Errorf("expected %v, got %v", wWith, w.with)
	}
	wGraphs := []Graph{{}}
	w.SetGraphs(wGraphs)
	if len(w.graphs) != 1 {
		t.Errorf("expected %v, got %v", wGraphs, w.graphs)
	}
}
