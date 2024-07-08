package workflow

import (
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
)

func TestNewWorkflow(t *testing.T) {
	id := id.NewWorkflowID()
	name := "Test Workflow"
	entryGraphId := "graph1"
	with := map[string]interface{}{"key": "value"}
	graphs := []Graph{{}}

	w := NewWorkflow(id, name, entryGraphId, with, graphs)

	want := &Workflow{
		id:           id,
		name:         name,
		entryGraphId: entryGraphId,
		with:         with,
		graphs:       graphs,
	}

	assert.Equal(t, w, want)
}
