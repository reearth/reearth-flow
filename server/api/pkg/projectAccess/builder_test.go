package projectAccess

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestBuilder_New(t *testing.T) {
	var tb = New()
	assert.NotNil(t, tb)
}

func TestBuilder_Build(t *testing.T) {
	paid := NewID()

	type args struct {
		id ID
	}

	tests := []struct {
		Name     string
		Args     args
		Expected *ProjectAccess
		Err      error
	}{
		{
			Name: "fail nil id",
			Args: args{
				id: ID{},
			},
			Err: ErrInvalidID,
		},
		{
			Name: "success build new project access",
			Args: args{
				id: paid,
			},
			Expected: &ProjectAccess{
				id: paid,
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.Name, func(t *testing.T) {
			t.Parallel()
			res, err := New().
				ID(tt.Args.id).
				Build()

			if tt.Err == nil {
				assert.Equal(t, tt.Expected, res)
			} else {
				assert.Equal(t, tt.Err, err)
			}
		})
	}
}

func TestBuilder_MustBuild(t *testing.T) {
	paid := NewID()

	type args struct {
		id ID
	}

	tests := []struct {
		Name     string
		Args     args
		Expected *ProjectAccess
		Err      error
	}{
		{
			Name: "fail nil id",
			Args: args{
				id: ID{},
			},
			Err: ErrInvalidID,
		},
		{
			Name: "success build new project access",
			Args: args{
				id: paid,
			},
			Expected: &ProjectAccess{
				id: paid,
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.Name, func(t *testing.T) {
			t.Parallel()

			build := func() *ProjectAccess {
				t.Helper()
				return New().
					ID(tt.Args.id).
					MustBuild()
			}

			if tt.Err != nil {
				assert.PanicsWithValue(t, tt.Err, func() { _ = build() })
			} else {
				assert.Equal(t, tt.Expected, build())
			}
		})
	}
}

func TestBuilder_ID(t *testing.T) {
	var tb = New()
	res := tb.ID(NewID()).MustBuild()
	assert.NotNil(t, res.ID())
}

func TestBuilder_NewID(t *testing.T) {
	var tb = New()
	res := tb.NewID().MustBuild()
	assert.NotNil(t, res.ID())
}

func TestBuilder_Project(t *testing.T) {
	var tb = New().NewID()
	pid := NewProjectID()
	res := tb.Project(pid).MustBuild()
	assert.Equal(t, pid, res.Project())
}

func TestBuilder_IsPublic(t *testing.T) {
	var tb = New().NewID()

	res := tb.IsPublic(true).MustBuild()
	assert.True(t, res.IsPublic())

	res = tb.IsPublic(false).MustBuild()
	assert.False(t, res.IsPublic())
}

func TestBuilder_Token(t *testing.T) {
	var tb = New().NewID()
	res := tb.Token("token").MustBuild()
	assert.Equal(t, "token", res.Token())
}

func TestBuilder_UserRoles(t *testing.T) {
	var tb = New().NewID()
	ur := NewUserRoleList()
	res := tb.UserRoles(ur).MustBuild()
	assert.Equal(t, ur, res.UserRoles())
}
