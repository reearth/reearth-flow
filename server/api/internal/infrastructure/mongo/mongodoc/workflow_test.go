package mongodoc

import (
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/idx"
	"github.com/stretchr/testify/assert"
)

func TestNewWorkflowConsumer(t *testing.T) {
	workspaces := []idx.ID[accountdomain.Workspace]{accountdomain.NewWorkspaceID(), accountdomain.NewWorkspaceID()}
	consumer := NewWorkflowConsumer(workspaces)

	assert.NotNil(t, consumer)
}

func TestNewWorkflow(t *testing.T) {
	wf := workflow.NewWorkflow(id.NewWorkflowID(), id.NewProjectID(), accountdomain.NewWorkspaceID(), "workflow_url", id.NewGraphID())

	doc, wid := NewWorkflow(wf)

	assert.Equal(t, wf.ID().String(), doc.ID)
	assert.Equal(t, wf.Project().String(), doc.Project)
	assert.Equal(t, wf.Workspace().String(), doc.Workspace)
	assert.Equal(t, wf.URL(), doc.URL)
	assert.Equal(t, wf.Graph().String(), doc.Graph)
	assert.Equal(t, wf.ID().String(), wid)
}

func TestWorkflowDocument_Model(t *testing.T) {
	expectedID := id.NewWorkflowID()
	expectedProject := id.NewProjectID()
	expectedWorkspace := accountdomain.NewWorkspaceID()
	expectedGraph := id.NewGraphID()
	url := "workflow_url"

	doc := &WorkflowDocument{
		ID:        expectedID.String(),
		Project:   expectedProject.String(),
		Workspace: expectedWorkspace.String(),
		Graph:     expectedGraph.String(),
		URL:       url,
	}

	model, err := doc.Model()

	assert.NoError(t, err)
	assert.Equal(t, expectedID, model.ID())
	assert.Equal(t, expectedProject, model.Project())
	assert.Equal(t, expectedWorkspace, model.Workspace())
	assert.Equal(t, expectedGraph, model.Graph())
	assert.Equal(t, url, model.URL())
}
