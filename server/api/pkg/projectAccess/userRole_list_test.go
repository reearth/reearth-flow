package projectAccess

import (
	"testing"

	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/stretchr/testify/assert"
)

func TestUserRoleList_Add(t *testing.T) {
	userID1 := user.NewID()
	userID2 := user.NewID()

	list := NewUserRoleList()

	// Add a user for the first time (success case)
	err := list.Add(userID1, workspace.RoleOwner)
	assert.NoError(t, err)
	assert.Len(t, list, 1)
	assert.Equal(t, userID1, list[0].UserID())
	assert.Equal(t, workspace.RoleOwner, list[0].Role())

	// Add another user (success case)
	err = list.Add(userID2, workspace.RoleWriter)
	assert.NoError(t, err)
	assert.Len(t, list, 2)
	assert.Equal(t, userID2, list[1].UserID())
	assert.Equal(t, workspace.RoleWriter, list[1].Role())

	// Add a duplicate user (failure case)
	err = list.Add(userID1, workspace.RoleReader)
	assert.Error(t, err)
	assert.Equal(t, ErrUserRoleExists, err)
	assert.Len(t, list, 2)
}

func TestUserRoleList_Edit(t *testing.T) {
	userID1 := user.NewID()
	userID2 := user.NewID()
	nonExistUserID := user.NewID()

	list := NewUserRoleList()
	_ = list.Add(userID1, workspace.RoleReader)
	_ = list.Add(userID2, workspace.RoleReader)

	// Update the role of an existing user (success case)
	err := list.Edit(userID1, workspace.RoleWriter)
	assert.NoError(t, err)
	assert.Equal(t, workspace.RoleWriter, list[0].Role())

	// Update the role of an existing user to the same value (failure case - no change)
	err = list.Edit(userID1, workspace.RoleWriter)
	assert.Error(t, err)
	assert.Equal(t, ErrNoRoleChange, err)
	assert.Equal(t, workspace.RoleWriter, list[0].Role())

	// Update the role of another existing user (success case)
	err = list.Edit(userID2, workspace.RoleMaintainer)
	assert.NoError(t, err)
	assert.Equal(t, workspace.RoleMaintainer, list[1].Role())

	// Update the role of a non-existent user (failure case)
	err = list.Edit(nonExistUserID, workspace.RoleWriter)
	assert.Error(t, err)
	assert.Equal(t, ErrUserRoleNotFound, err)
}

func TestUserRoleList_Remove(t *testing.T) {
	userID1 := user.NewID()
	userID2 := user.NewID()
	userID3 := user.NewID()
	nonExistUserID := user.NewID()

	list := NewUserRoleList()
	_ = list.Add(userID1, workspace.RoleOwner)
	_ = list.Add(userID2, workspace.RoleWriter)
	_ = list.Add(userID3, workspace.RoleReader)

	assert.Len(t, list, 3)

	// Remove the first user (success case)
	err := list.Remove(userID1)
	assert.NoError(t, err)
	assert.Len(t, list, 2)

	for _, ur := range list {
		assert.NotEqual(t, userID1, ur.UserID())
	}

	// Remove the middle user (success case)
	err = list.Remove(userID2)
	assert.NoError(t, err)
	assert.Len(t, list, 1)
	assert.Equal(t, userID3, list[0].UserID())

	// Remove a non-existent user (failure case)
	err = list.Remove(nonExistUserID)
	assert.Error(t, err)
	assert.Equal(t, ErrUserRoleNotFound, err)
	assert.Len(t, list, 1)

	// Remove the last user (success case)
	err = list.Remove(userID3)
	assert.NoError(t, err)
	assert.Empty(t, list)
}
