package repo

import (
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/user"
	"github.com/reearth/reearth-flow/api/pkg/workspace"
	"github.com/stretchr/testify/assert"
)

func TestWorkspaceFilter_Merge(t *testing.T) {
	a := workspace.NewID()
	b := workspace.NewID()
	assert.Equal(t, WorkspaceFilter{
		Readable: id.WorkspaceIDList{a, b},
		Writable: id.WorkspaceIDList{b, a},
	}, WorkspaceFilter{
		Readable: id.WorkspaceIDList{a},
		Writable: id.WorkspaceIDList{b},
	}.Merge(WorkspaceFilter{
		Readable: id.WorkspaceIDList{b},
		Writable: id.WorkspaceIDList{a},
	}))
	assert.Equal(t, WorkspaceFilter{Readable: id.WorkspaceIDList{}}, WorkspaceFilter{}.Merge(WorkspaceFilter{Readable: user.WorkspaceIDList{}}))
	assert.Equal(t, WorkspaceFilter{Readable: id.WorkspaceIDList{}}, WorkspaceFilter{Readable: user.WorkspaceIDList{}}.Merge(WorkspaceFilter{}))
	assert.Equal(t, WorkspaceFilter{Writable: id.WorkspaceIDList{}}, WorkspaceFilter{}.Merge(WorkspaceFilter{Writable: user.WorkspaceIDList{}}))
	assert.Equal(t, WorkspaceFilter{Writable: id.WorkspaceIDList{}}, WorkspaceFilter{Writable: user.WorkspaceIDList{}}.Merge(WorkspaceFilter{}))
}
