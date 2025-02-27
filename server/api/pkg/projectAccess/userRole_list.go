package projectAccess

import (
	"errors"

	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
)

var (
	ErrUserRoleNotFound = errors.New("user role not found")
	ErrUserRoleExists   = errors.New("user role already exists")
	ErrNoRoleChange     = errors.New("role is already set to the requested value")
)

type UserRoleList []UserRole

func NewUserRoleList() UserRoleList {
	return UserRoleList{}
}

func (l *UserRoleList) Add(userID user.ID, role workspace.Role) error {
	for _, ur := range *l {
		if ur.userID == userID {
			return ErrUserRoleExists
		}
	}

	*l = append(*l, UserRole{
		userID: userID,
		role:   role,
	})
	return nil
}

func (l *UserRoleList) Edit(userID user.ID, role workspace.Role) error {
	for i, ur := range *l {
		if ur.userID == userID {
			if ur.role == role {
				return ErrNoRoleChange
			}
			(*l)[i].role = role
			return nil
		}
	}
	return ErrUserRoleNotFound
}

func (l *UserRoleList) Remove(userID user.ID) error {
	for i, ur := range *l {
		if ur.userID == userID {
			// Remove by replacing with the last element and shrinking the slice
			(*l)[i] = (*l)[len(*l)-1]
			*l = (*l)[:len(*l)-1]
			return nil
		}
	}
	return ErrUserRoleNotFound
}
