package repo

import (
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
	"github.com/stretchr/testify/assert"
)

func TestWorkspaceFilter_Merge(t *testing.T) {
	a := accountsworkspace.NewID()
	b := accountsworkspace.NewID()
	assert.Equal(t, WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{a, b},
		Writable: accountsid.WorkspaceIDList{b, a},
	}, WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{a},
		Writable: accountsid.WorkspaceIDList{b},
	}.Merge(WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{b},
		Writable: accountsid.WorkspaceIDList{a},
	}))
	assert.Equal(t, WorkspaceFilter{Readable: accountsid.WorkspaceIDList{}}, WorkspaceFilter{}.Merge(WorkspaceFilter{Readable: accountsuser.WorkspaceIDList{}}))
	assert.Equal(t, WorkspaceFilter{Readable: accountsid.WorkspaceIDList{}}, WorkspaceFilter{Readable: accountsuser.WorkspaceIDList{}}.Merge(WorkspaceFilter{}))
	assert.Equal(t, WorkspaceFilter{Writable: accountsid.WorkspaceIDList{}}, WorkspaceFilter{}.Merge(WorkspaceFilter{Writable: accountsuser.WorkspaceIDList{}}))
	assert.Equal(t, WorkspaceFilter{Writable: accountsid.WorkspaceIDList{}}, WorkspaceFilter{Writable: accountsuser.WorkspaceIDList{}}.Merge(WorkspaceFilter{}))
}
