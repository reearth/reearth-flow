package gqlmodel

import (
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/workspace"
	"github.com/stretchr/testify/assert"
)

func TestFromRole(t *testing.T) {
	assert.Equal(t, workspace.RoleOwner, FromRole(RoleOwner))
	assert.Equal(t, workspace.RoleMaintainer, FromRole(RoleMaintainer))
	assert.Equal(t, workspace.RoleWriter, FromRole(RoleWriter))
	assert.Equal(t, workspace.RoleReader, FromRole(RoleReader))
	assert.Equal(t, workspace.Role(""), FromRole("unknown"))
}
