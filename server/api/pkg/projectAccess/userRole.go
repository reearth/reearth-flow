package projectAccess

import (
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
)

type UserRole struct {
	userID user.ID
	role   workspace.Role
}

func (ur UserRole) UserID() user.ID {
	return ur.userID
}

func (ur UserRole) Role() workspace.Role {
	return ur.role
}
