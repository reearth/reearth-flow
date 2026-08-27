package interactor

import (
	"testing"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestProjectRunActionAdmitsReadersAndWriters pins the role boundary for a
// debug run. Project.Run authorizes with ActionRead; gating it on ActionEdit
// (maintainer/owner) or ActionAny (writer and above) excludes the roles that
// are meant to be able to run one.
//
// This asserts against DefineResources(), the same input the Cerbos policies
// are generated from, so it fails if the roles for that action are narrowed.
func TestProjectRunActionAdmitsReadersAndWriters(t *testing.T) {
	var runRoles, anyRoles []string
	for _, r := range rbac.DefineResources() {
		if r.Resource != rbac.ResourceProject {
			continue
		}
		runRoles = r.Actions[rbac.ActionRead].Roles
		anyRoles = r.Actions[rbac.ActionAny].Roles
	}
	require.NotEmpty(t, runRoles, "ResourceProject must declare the action Run authorizes with")

	for _, want := range []string{"reader", "writer", "maintainer", "owner"} {
		assert.Contains(t, runRoles, want, "%s must be able to trigger a debug run", want)
	}

	// Run cannot sit on ActionAny: that rule deliberately excludes reader
	// because it also gates the editor's save.
	assert.NotContains(t, anyRoles, "reader",
		"ActionAny gates FlushToGCS; adding reader there would let readers persist writes")
}
