package project

import (
	"testing"
	"time"

	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/stretchr/testify/assert"
)

func TestProject_SetArchived(t *testing.T) {
	p := &Project{isArchived: false}
	p.SetArchived(true)
	assert.Equal(t, true, p.IsArchived())
}

func TestProject_SetUpdatedAt(t *testing.T) {
	p := &Project{}
	p.SetUpdatedAt(time.Date(1900, 1, 1, 00, 00, 1, 1, time.UTC))
	assert.Equal(t, time.Date(1900, 1, 1, 00, 00, 1, 1, time.UTC), p.UpdatedAt())
}

func TestProject_UpdateName(t *testing.T) {
	p := &Project{}
	p.UpdateName("foo")
	assert.Equal(t, "foo", p.Name())
}

func TestProject_UpdateDescription(t *testing.T) {
	p := &Project{}
	p.UpdateDescription("aaa")
	assert.Equal(t, "aaa", p.Description())
}

func TestProject_UpdateWorkspace(t *testing.T) {
	p := &Project{}
	p.UpdateWorkspace(accountdomain.NewWorkspaceID())
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
