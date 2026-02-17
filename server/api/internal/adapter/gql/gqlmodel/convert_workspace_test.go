package gqlmodel

import (
	"testing"

	"github.com/reearth/reearth-accounts/server/pkg/role"
	"github.com/stretchr/testify/assert"
)

func TestFromRole(t *testing.T) {
	assert.Equal(t, role.RoleOwner, FromRole(RoleOwner))
	assert.Equal(t, role.RoleMaintainer, FromRole(RoleMaintainer))
	assert.Equal(t, role.RoleWriter, FromRole(RoleWriter))
	assert.Equal(t, role.RoleReader, FromRole(RoleReader))
	assert.Equal(t, role.RoleType(""), FromRole("unknown"))
}
