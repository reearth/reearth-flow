package workflow

import (
	"reflect"
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
	"gopkg.in/yaml.v2"
)

func TestNewWorkflow(t *testing.T) {
	workspaceID := NewWorkspaceID()
	projectID := id.NewProjectID()
	workflowID := id.NewWorkflowID()

	result := NewWorkflow(workflowID, projectID, workspaceID, nil)

	want := &Workflow{
		ID:         workflowID,
		Project:    projectID,
		Workspace:  workspaceID,
		YamlString: nil,
	}

	assert.Equal(t, result, want)
}

func TestToWorkflowYaml(t *testing.T) {
	wfid := id.NewWorkflowID()
	name := "Test Workflow"
	entryGraphID := "entryGraph123"
	with := map[string]interface{}{
		"key1": "value1",
		"key2": 2,
	}
	graphID1 := NewGraphID()
	graphID2 := NewGraphID()
	graphs := []*Graph{
		{
			ID:   graphID1,
			Name: "Graph One",
		},
		{
			ID:   graphID2,
			Name: "Graph Two",
		},
	}

	expected := map[string]interface{}{
		"id":           wfid.String(),
		"entryGraphID": entryGraphID,
		"with":         &with,
		"graphs":       graphs,
	}

	expectedYaml, err := yaml.Marshal(expected)
	if err != nil {
		t.Fatalf("Failed to marshal expected value: %v", err)
	}

	expectedString := string(expectedYaml)
	result := ToWorkflowYaml(wfid, name, entryGraphID, &with, graphs)

	if result == nil {
		t.Fatalf("Expected non-nil result, got nil")
	}

	if !reflect.DeepEqual(expectedString, *result) {
		t.Errorf("Expected %s, got %s", expectedString, *result)
	}
}
