package interactor

import (
	"testing"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestProjectRunActionAdmitsWritersNotReaders(t *testing.T) {
	var runRoles []string
	for _, r := range rbac.DefineResources() {
		if r.Resource == rbac.ResourceProject {
			runRoles = r.Actions[rbac.ActionAny].Roles
		}
	}
	require.NotEmpty(t, runRoles, "ResourceProject must declare the action Run authorizes with")

	for _, want := range []string{"writer", "maintainer", "owner"} {
		assert.Contains(t, runRoles, want, "%s must be able to trigger a debug run", want)
	}
	assert.NotContains(t, runRoles, "reader", "readers cannot trigger a debug run")
}
