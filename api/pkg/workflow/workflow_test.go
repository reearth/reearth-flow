package workflow

import "testing"

func TestWorkflow_ID(t *testing.T) {
	wID := NewID()
	w := &Workflow{id: wID}
	if w.ID() != wID {
		t.Errorf("TestWorkflow_ID failed")
	}
}

func TestWorkflow_SetNodes(t *testing.T) {
	w := &Workflow{}
	w.SetNodes([]NodeType{{}})
	if len(w.nodes) != 1 {
		t.Errorf("TestWorkflow_SetNodes failed")
	}
}

func TestWorkflow_SetEdges(t *testing.T) {
	w := &Workflow{}
	w.SetEdges([]Edges{{}})
	if len(w.edges) != 1 {
		t.Errorf("TestWorkflow_SetEdges failed")
	}
}

func TestWorkflow_SetIsMain(t *testing.T) {
	w := &Workflow{}
	w.SetIsMain(true)
	if w.isMain != true {
		t.Errorf("TestWorkflow_SetIsMain failed")
	}
}

func TestWorkflow_SetProjectVersion(t *testing.T) {
	w := &Workflow{}
	w.SetProjectVersion(1)
	if w.projectVersion != 1 {
		t.Errorf("TestWorkflow_SetProjectVersion failed")
	}
}

func TestWorkflow_SetProjectID(t *testing.T) {
	w := &Workflow{}
	pID := NewProjectID()
	w.SetProjectID(pID)
	if w.projectID != pID {
		t.Errorf("TestWorkflow_SetProjectID failed")
	}
}

func TestWorkflow_SetWorkspaceID(t *testing.T) {
	w := &Workflow{}
	wID := NewWorkspaceID()
	w.SetWorkspaceID(wID)
	if w.workspaceID != wID {
		t.Errorf("TestWorkflow_SetWorkspaceID failed")
	}
}
