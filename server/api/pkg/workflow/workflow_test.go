package workflow

import (
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
)

func TestNewWorkflow(t *testing.T) {
	workspaceID := NewWorkspaceID()
	projectID := id.NewProjectID()
	workflowID := id.NewWorkflowID()
	url := "http://example.com"
	graphID := id.NewGraphID()

	result := NewWorkflow(workflowID, projectID, workspaceID, url, graphID)

	want := &Workflow{
		id:        workflowID,
		project:   projectID,
		workspace: workspaceID,
		url:       url,
		graph:     graphID,
	}

	assert.Equal(t, result, want)
}

func TestWorkflowGetters(t *testing.T) {
	workspaceID := NewWorkspaceID()
	projectID := id.NewProjectID()
	workflowID := id.NewWorkflowID()
	url := "http://example.net"
	graphID := id.NewGraphID()

	w := NewWorkflow(workflowID, projectID, workspaceID, url, graphID)

	assert.Equal(t, workflowID, w.ID(), "ID getter should return the correct workflow ID")
	assert.Equal(t, projectID, w.Project(), "Project getter should return the correct project ID")
	assert.Equal(t, workspaceID, w.Workspace(), "Workspace getter should return the correct workspace ID")
	assert.Equal(t, url, w.URL(), "URL getter should return the correct URL")
	assert.Equal(t, graphID, w.Graph(), "Graph getter should return the correct graph ID")
}
