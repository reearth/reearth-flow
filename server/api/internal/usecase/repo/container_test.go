package repo

import (
	"testing"

	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/stretchr/testify/assert"
)

func TestWorkspaceFilter_Merge(t *testing.T) {
	a := workspace.NewID()
	b := workspace.NewID()
	assert.Equal(t, WorkspaceFilter{
		Readable: accountdomain.WorkspaceIDList{a, b},
		Writable: accountdomain.WorkspaceIDList{b, a},
	}, WorkspaceFilter{
		Readable: accountdomain.WorkspaceIDList{a},
		Writable: accountdomain.WorkspaceIDList{b},
	}.Merge(WorkspaceFilter{
		Readable: accountdomain.WorkspaceIDList{b},
		Writable: accountdomain.WorkspaceIDList{a},
	}))
	assert.Equal(t, WorkspaceFilter{Readable: accountdomain.WorkspaceIDList{}}, WorkspaceFilter{}.Merge(WorkspaceFilter{Readable: user.WorkspaceIDList{}}))
	assert.Equal(t, WorkspaceFilter{Readable: accountdomain.WorkspaceIDList{}}, WorkspaceFilter{Readable: user.WorkspaceIDList{}}.Merge(WorkspaceFilter{}))
	assert.Equal(t, WorkspaceFilter{Writable: accountdomain.WorkspaceIDList{}}, WorkspaceFilter{}.Merge(WorkspaceFilter{Writable: user.WorkspaceIDList{}}))
	assert.Equal(t, WorkspaceFilter{Writable: accountdomain.WorkspaceIDList{}}, WorkspaceFilter{Writable: user.WorkspaceIDList{}}.Merge(WorkspaceFilter{}))
}
