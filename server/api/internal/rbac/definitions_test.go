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
func TestProjectPolicyCoversTheDocumentActions(t *testing.T) {
	actions := actionsFor(t, ResourceProject)

	for _, action := range []string{ActionAny, ActionEdit, ActionDelete} {
		require.Contains(t, actions, action,
			"interactor.Websocket checks %q on %s; with no rule it is denied for everyone",
			action, ResourceProject)
	}

	// Reads and save use ActionAny, so it must reach everyone who can see the
	// project — including writers, who would otherwise be unable to save.
	assert.Contains(t, actions[ActionAny].Roles, roleWriter,
		"writers must be able to save; saveSnapshot authorizes with ActionAny")
	assert.Contains(t, actions[ActionAny].Roles, roleReader,
		"readers must be able to view version history")

	// ActionRead is deliberately NOT used: flow:project has no read rule, so
	// checking it would deny every user. Guard against a well-meaning switch back.
	assert.NotContains(t, actions, ActionRead,
		"if a read rule is added here, revisit interactor.Websocket, which avoids ActionRead precisely because there is none")
}

// TestProjectDocumentPolicyStaysDeclared keeps the target model intact so the
// move back is one line. See the TODO in interactor.Websocket.
func TestProjectDocumentPolicyStaysDeclared(t *testing.T) {
	actions := actionsFor(t, ResourceProjectDocument)
	assert.Contains(t, actions, ActionRead)
	assert.Contains(t, actions[ActionEdit].Roles, roleWriter,
		"the target model exists to let writers mutate a document without project-settings rights")
}
