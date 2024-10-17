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
		ID:        workflowID,
		Project:   projectID,
		Workspace: workspaceID,
		URL:       url,
	}

	assert.Equal(t, result, want)
}
