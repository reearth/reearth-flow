package rbac

import (
	"testing"

	"github.com/reearth/reearthx/cerbos/generator"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// actionsFor returns the declared action rules for one resource.
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

// TestProjectDocumentPolicy pins the rules the document operations depend on.
//
// This is the check that was missing. The interactor tests use a mock permission
// checker, so they prove the right resource and action are REQUESTED; only the
// policy decides whether that request can ever be granted. An action with no rule
// is denied by default, so a read operation checking an undeclared action would
// fail for every user including owners, and no unit test would notice.
func TestProjectDocumentPolicy(t *testing.T) {
	actions := actionsFor(t, ResourceProjectDocument)

	// Every action the Websocket interactor checks must have a rule.
	for _, action := range []string{ActionRead, ActionEdit, ActionDelete} {
		require.Contains(t, actions, action,
			"interactor.Websocket checks %q on %s; without a rule it is denied for everyone",
			action, ResourceProjectDocument)
	}

	// Reading version history is for anyone who can see the project.
	assert.ElementsMatch(t, []string{roleReader, roleWriter, roleMaintainer, roleOwner},
		actions[ActionRead].Roles)

	// Editing the document is content work, so writers are included. Excluding
	// them would stop the people doing the collaborative editing from saving a
	// version, rolling back, importing or copying.
	assert.Contains(t, actions[ActionEdit].Roles, roleWriter,
		"writers must be able to mutate a project's document")
	assert.NotContains(t, actions[ActionEdit].Roles, roleReader,
		"readers must not be able to mutate a project's document")

	// Destroying all of a project's document data stays privileged.
	assert.ElementsMatch(t, []string{roleMaintainer, roleOwner}, actions[ActionDelete].Roles)
}

// TestProjectPolicyUnchangedByDocumentSplit guards the reason the document
// operations got their own resource: ResourceProject's edit is project SETTINGS
// (rename, configure) and is deliberately maintainer and owner only. Adding
// writers there to unblock document editing would have widened that too.
func TestProjectPolicyUnchangedByDocumentSplit(t *testing.T) {
	actions := actionsFor(t, ResourceProject)
	assert.NotContains(t, actions[ActionEdit].Roles, roleWriter,
		"project settings edit must stay maintainer/owner; document editing lives on %s", ResourceProjectDocument)
}
