package rbac

import (
	"testing"

	"github.com/reearth/reearthx/cerbos/generator"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func actionsFor(t *testing.T, resource string) map[string]generator.ActionRule {
	t.Helper()
	for _, r := range DefineResources() {
		if r.Resource == resource {
			return r.Actions
		}
	}
	t.Fatalf("resource %q is not declared in DefineResources", resource)
	return nil
}

// The interactor tests use a mock checker, so only this test proves the policy
// grants the actions interactor.Websocket requests. ActionAny gates FlushToGCS,
// a write, so readers must get their document access through ActionRead instead.
func TestProjectPolicyCoversTheDocumentActions(t *testing.T) {
	actions := actionsFor(t, ResourceProject)

	for _, action := range []string{ActionAny, ActionRead, ActionEdit, ActionDelete} {
		require.Contains(t, actions, action,
			"interactor.Websocket checks %q on %s; with no rule it is denied for everyone",
			action, ResourceProject)
	}

	assert.Contains(t, actions[ActionAny].Roles, roleWriter,
		"writers must be able to save; FlushToGCS authorizes with ActionAny")
	assert.NotContains(t, actions[ActionAny].Roles, roleReader,
		"ActionAny also gates FlushToGCS (a write); reader must not be granted it")

	assert.Contains(t, actions[ActionRead].Roles, roleReader,
		"readers must be able to view version history")
	assert.Contains(t, actions[ActionRead].Roles, roleWriter,
		"writers must retain read access alongside readers")
}

// TestProjectDocumentPolicyStaysDeclared keeps the unused projectDocument model
// declared, so moving interactor.Websocket back to it is a one-line change.
func TestProjectDocumentPolicyStaysDeclared(t *testing.T) {
	actions := actionsFor(t, ResourceProjectDocument)
	assert.Contains(t, actions, ActionRead)
	assert.Contains(t, actions[ActionEdit].Roles, roleWriter,
		"the target model exists to let writers mutate a document without project-settings rights")
}
