package projectAccess

import (
	"testing"

	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/stretchr/testify/assert"
)

func TestUserRole_UserID(t *testing.T) {
	expectedUserID := user.NewID()
	ur := &UserRole{userID: expectedUserID}
	assert.Equal(t, expectedUserID, ur.UserID())
}

func TestUserRole_Role(t *testing.T) {
	expectedRole := workspace.RoleOwner
	ur := &UserRole{role: expectedRole}
	assert.Equal(t, expectedRole, ur.Role())
}
