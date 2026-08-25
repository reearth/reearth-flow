package interactor

import (
	"testing"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestProjectRunActionAdmitsWriters pins the role boundary for a debug run.
// Project.Run authorizes with ActionAny; gating it on ActionEdit instead
// (maintainer/owner) is what stopped writers running the editor's debug run.
//
// This asserts against DefineResources(), the same input the Cerbos policies
// are generated from, so it fails if the roles for that action are narrowed.
func TestProjectRunActionAdmitsWriters(t *testing.T) {
	var runRoles, editRoles []string
	for _, r := range rbac.DefineResources() {
		if r.Resource != rbac.ResourceProject {
			continue
		}
		runRoles = r.Actions[rbac.ActionAny].Roles
		editRoles = r.Actions[rbac.ActionEdit].Roles
	}
	require.NotEmpty(t, runRoles, "ResourceProject must declare the action Run authorizes with")

	assert.Contains(t, runRoles, "writer", "writers must be able to trigger a debug run")
	assert.Contains(t, runRoles, "maintainer")
	assert.Contains(t, runRoles, "owner")
	assert.NotContains(t, runRoles, "reader", "a debug run executes the workflow; readers are not granted it")

	assert.NotContains(t, editRoles, "writer",
		"ActionEdit stays maintainer/owner: if it ever admits writers, Run should move back to it")
}
