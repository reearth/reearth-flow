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

	result := NewWorkflow(workflowID, projectID, workspaceID, url)

	want := &Workflow{
		id:        workflowID,
		project:   projectID,
		workspace: workspaceID,
		url:       url,
	}

	assert.Equal(t, result, want)
}

func TestWorkflowGetters(t *testing.T) {
	workspaceID := NewWorkspaceID()
	projectID := id.NewProjectID()
	workflowID := id.NewWorkflowID()
	url := "http://example.net"

	w := NewWorkflow(workflowID, projectID, workspaceID, url)

	assert.Equal(t, workflowID, w.ID(), "ID getter should return the correct workflow ID")
	assert.Equal(t, projectID, w.Project(), "Project getter should return the correct project ID")
	assert.Equal(t, workspaceID, w.Workspace(), "Workspace getter should return the correct workspace ID")
	assert.Equal(t, url, w.URL(), "URL getter should return the correct URL")
}
