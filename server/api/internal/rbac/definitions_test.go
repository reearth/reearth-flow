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

// TestProjectPolicyCoversTheDocumentActions is the check whose absence caused a
// production break.
//
// Cerbos denies any action with no matching rule, so an action the code checks
// but the policy does not declare is denied for EVERYONE, owners included. The
// interactor tests cannot catch it: they use a mock checker, so they only prove
// the right action is requested, never that it can be granted.
//
// interactor.Websocket checks ResourceProject with these actions.
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

// TestProjectDocumentPolicyStaysDeclared: the resource is kept so that once the
// policy store is refreshed from generated output, moving back to it is a
// one-line change. See the TODO in interactor.Websocket.
func TestProjectDocumentPolicyStaysDeclared(t *testing.T) {
	actions := actionsFor(t, ResourceProjectDocument)
	assert.Contains(t, actions, ActionRead)
	assert.Contains(t, actions[ActionEdit].Roles, roleWriter,
		"the target model exists to let writers mutate a document without project-settings rights")
}
