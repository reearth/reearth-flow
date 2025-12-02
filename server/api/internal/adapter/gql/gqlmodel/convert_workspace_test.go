package gqlmodel

import (
	"testing"

	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
	"github.com/stretchr/testify/assert"
)

func TestFromRole(t *testing.T) {
	assert.Equal(t, accountsworkspace.RoleOwner, FromRole(RoleOwner))
	assert.Equal(t, accountsworkspace.RoleMaintainer, FromRole(RoleMaintainer))
	assert.Equal(t, accountsworkspace.RoleWriter, FromRole(RoleWriter))
	assert.Equal(t, accountsworkspace.RoleReader, FromRole(RoleReader))
	assert.Equal(t, accountsworkspace.Role(""), FromRole("unknown"))
}
