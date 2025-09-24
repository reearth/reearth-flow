package gqlmodel

import (
	"testing"

	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/stretchr/testify/assert"
)

func TestToRole(t *testing.T) {
	assert.Equal(t, Role(RoleOwner), ToRoleFromReearthx(workspace.RoleOwner))
	assert.Equal(t, Role(RoleMaintainer), ToRoleFromReearthx(workspace.RoleMaintainer))
	assert.Equal(t, Role(RoleWriter), ToRoleFromReearthx(workspace.RoleWriter))
	assert.Equal(t, Role(RoleReader), ToRoleFromReearthx(workspace.RoleReader))
	assert.Equal(t, Role(""), ToRoleFromReearthx(workspace.Role("unknown")))
}

func TestFromRole(t *testing.T) {
	assert.Equal(t, workspace.RoleOwner, FromRoleToReearthx(RoleOwner))
	assert.Equal(t, workspace.RoleMaintainer, FromRoleToReearthx(RoleMaintainer))
	assert.Equal(t, workspace.RoleWriter, FromRoleToReearthx(RoleWriter))
	assert.Equal(t, workspace.RoleReader, FromRoleToReearthx(RoleReader))
	assert.Equal(t, workspace.Role(""), FromRoleToReearthx("unknown"))
}
