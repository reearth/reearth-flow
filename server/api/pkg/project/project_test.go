package project

import (
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/stretchr/testify/assert"
)

func TestProject_SetArchived(t *testing.T) {
	p := &Project{isArchived: false}
	p.SetArchived(true)
	assert.Equal(t, true, p.IsArchived())
}

func TestProject_SetUpdateName(t *testing.T) {
	p := &Project{}
	p.SetUpdateName("foo")
	assert.Equal(t, "foo", p.Name())
}

func TestProject_SetUpdateDescription(t *testing.T) {
	p := &Project{}
	p.SetUpdateDescription("aaa")
	assert.Equal(t, "aaa", p.Description())
}

func TestProject_SetUpdateWorkspace(t *testing.T) {
	p := &Project{}
	p.SetUpdateWorkspace(accountsid.NewWorkspaceID())
	assert.NotNil(t, p.Workspace())
}

func TestProject_IsBasicAuthActive(t *testing.T) {
	tests := []struct {
		name     string
		p        *Project
		expected bool
	}{
		{
			name: "basic auth is inactive",
			p: &Project{
				isBasicAuthActive: false,
			},
			expected: false,
		},
		{
			name: "basic auth is active",
			p: &Project{
				isBasicAuthActive: true,
			},
			expected: true,
		},
	}

	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			t.Parallel()
			res := tt.p.IsBasicAuthActive()
			assert.Equal(t, tt.expected, res)
		})
	}
}

func TestProject_SetIsLocked(t *testing.T) {
	t.Parallel()

	p := &Project{}
	p.SetIsLocked(true)
	assert.True(t, p.IsLocked())
}

func TestProject_SetIsLocked_OrdersAfterAClockThatRanAhead(t *testing.T) {
	t.Parallel()

	// Saved by a replica whose clock runs an hour ahead of this one.
	ahead := time.Now().Add(time.Hour).Truncate(time.Millisecond)
	p := New().NewID().Workspace(NewWorkspaceID()).UpdatedAt(ahead).MustBuild()

	p.SetIsLocked(true)
	assert.True(t, p.UpdatedAt().Equal(ahead.Add(time.Millisecond)), "updatedAt = %v", p.UpdatedAt())
	p.SetIsLocked(false)
	assert.True(t, p.UpdatedAt().Equal(ahead.Add(2*time.Millisecond)), "updatedAt = %v", p.UpdatedAt())
}

func TestProject_RecordEdit(t *testing.T) {
	t.Parallel()

	base := time.Date(2026, 11, 18, 3, 0, 0, 0, time.UTC)
	metadataEditor := accountsid.NewUserID()
	canvasEditor := accountsid.NewUserID()
	newProject := func() *Project {
		return New().NewID().Workspace(NewWorkspaceID()).UpdatedAt(base).UpdatedBy(&metadataEditor).MustBuild()
	}

	t.Run("a later edit moves updatedAt and updatedBy", func(t *testing.T) {
		t.Parallel()
		p := newProject()

		assert.True(t, p.RecordEdit(base.Add(time.Second), &canvasEditor))
		assert.True(t, p.UpdatedAt().Equal(base.Add(time.Second)))
		assert.Equal(t, &canvasEditor, p.UpdatedBy())
	})

	t.Run("an older edit changes nothing", func(t *testing.T) {
		t.Parallel()
		p := newProject()

		assert.False(t, p.RecordEdit(base.Add(-time.Second), &canvasEditor))
		assert.True(t, p.UpdatedAt().Equal(base))
		assert.Equal(t, &metadataEditor, p.UpdatedBy())
	})

	t.Run("an edit at the same time changes nothing", func(t *testing.T) {
		t.Parallel()
		p := newProject()

		assert.False(t, p.RecordEdit(base, &canvasEditor))
		assert.Equal(t, &metadataEditor, p.UpdatedBy())
	})

	t.Run("an unknown editor clears updatedBy", func(t *testing.T) {
		t.Parallel()
		p := newProject()

		assert.True(t, p.RecordEdit(base.Add(time.Second), nil))
		assert.Nil(t, p.UpdatedBy())
	})
}

func TestProject_BasicAuthUsername(t *testing.T) {
	t.Run("return basic auth username", func(t *testing.T) {
		p := &Project{basicAuthUsername: "test1"}
		res := p.BasicAuthUsername()
		assert.Equal(t, "test1", res)
	})
}

func TestProject_BasicAuthPassword(t *testing.T) {
	t.Run("return basic auth password", func(t *testing.T) {
		p := &Project{basicAuthPassword: "password"}
		res := p.BasicAuthPassword()
		assert.Equal(t, "password", res)
	})
}

func TestProject_SetIsBasicAuthActive(t *testing.T) {
	p := &Project{}
	p.SetIsBasicAuthActive(true)
	assert.Equal(t, true, p.isBasicAuthActive)
}

func TestProject_SetBasicAuthUsername(t *testing.T) {
	p := &Project{}
	p.SetBasicAuthUsername("username")
	assert.Equal(t, "username", p.basicAuthUsername)
}

func TestProject_SetBasicAuthPassword(t *testing.T) {
	p := &Project{}
	p.SetBasicAuthPassword("password")
	assert.Equal(t, "password", p.basicAuthPassword)
}
