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

// TestProjectPolicyCoversTheDocumentActions: an action with no rule is denied for
// everyone, owners included. The interactor tests use a mock checker, so they
// prove the action is requested but never that the policy grants it.
//
// ActionAny and ActionRead must NOT grant the same role sets. ActionAny also
// gates FlushToGCS — interactor.Websocket's editor-save write — so it must
// exclude reader; ActionRead gates the read-only document calls and must
// include reader. A previous version of this policy let ActionAny include
// reader (to satisfy "readers must see version history", since no ActionRead
// rule existed yet for this resource), which meant any reader could also call
// FlushToGCS and persist writes to a project they could only view. Do not
// reintroduce that: readers get their access through ActionRead now.
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

// TestProjectDocumentPolicyStaysDeclared keeps the target model intact so the
// move back is one line. See the TODO in interactor.Websocket.
func TestProjectDocumentPolicyStaysDeclared(t *testing.T) {
	actions := actionsFor(t, ResourceProjectDocument)
	assert.Contains(t, actions, ActionRead)
	assert.Contains(t, actions[ActionEdit].Roles, roleWriter,
		"the target model exists to let writers mutate a document without project-settings rights")
}
