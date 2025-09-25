package project

import (
	"reflect"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
)

func TestNew(t *testing.T) {
	tb := New()
	assert.NotNil(t, tb)
}

func TestBuilder_ID(t *testing.T) {
	tb := New()
	res := tb.ID(NewID()).MustBuild()
	assert.NotNil(t, res.ID())
}

func TestBuilder_Name(t *testing.T) {
	tb := New().NewID()
	res := tb.Name("foo").MustBuild()
	assert.Equal(t, "foo", res.Name())
}

func TestBuilder_NewID(t *testing.T) {
	tb := New()
	res := tb.NewID().MustBuild()
	assert.NotNil(t, res.ID())
}

func TestBuilder_Description(t *testing.T) {
	tb := New().NewID()
	res := tb.Description("desc").MustBuild()
	assert.Equal(t, "desc", res.Description())
}

func TestBuilder_IsArchived(t *testing.T) {
	tb := New().NewID()
	res := tb.IsArchived(true).MustBuild()
	assert.True(t, res.IsArchived())
}

func TestBuilder_BasicAuthUsername(t *testing.T) {
	tb := New().NewID()
	res := tb.BasicAuthUsername("username").MustBuild()
	assert.Equal(t, "username", res.BasicAuthUsername())
}

func TestBuilder_BasicAuthPassword(t *testing.T) {
	tb := New().NewID()
	res := tb.BasicAuthPassword("password").MustBuild()
	assert.Equal(t, "password", res.BasicAuthPassword())
}

func TestBuilder_Workspace(t *testing.T) {
	tb := New().NewID()
	res := tb.Workspace(id.NewWorkspaceID()).MustBuild()
	assert.NotNil(t, res.Workspace())
}

func TestBuilder_UpdatedAt(t *testing.T) {
	tb := New().NewID()
	d := time.Date(1900, 1, 1, 00, 0o0, 0, 1, time.UTC)
	res := tb.UpdatedAt(d).MustBuild()
	assert.True(t, reflect.DeepEqual(res.UpdatedAt(), d))
}

func TestBuilder_Build(t *testing.T) {
	d := time.Date(1900, 1, 1, 00, 0o0, 0, 1, time.UTC)
	pid := NewID()
	tid := id.NewWorkspaceID()

	type args struct {
		name, description string
		id                ID
		isArchived        bool
		updatedAt         time.Time
		workspace         WorkspaceID
	}

	tests := []struct {
		name     string
		args     args
		expected *Project
		err      error
	}{
		{
			name: "build normal project",
			args: args{
				name:        "xxx.aaa",
				description: "ddd",
				id:          pid,
				isArchived:  false,
				updatedAt:   d,
				workspace:   tid,
			},
			expected: &Project{
				id:          pid,
				description: "ddd",
				name:        "xxx.aaa",
				isArchived:  false,
				updatedAt:   d,
				workspace:   tid,
			},
		},
		{
			name: "zero updated at",
			args: args{
				id: pid,
			},
			expected: &Project{
				id:        pid,
				updatedAt: pid.Timestamp(),
			},
		},
		{
			name: "failed invalid id",
			err:  ErrInvalidID,
		},
	}

	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			t.Parallel()
			p, err := New().
				ID(tt.args.id).
				UpdatedAt(tt.args.updatedAt).
				Workspace(tt.args.workspace).
				Name(tt.args.name).
				UpdatedAt(tt.args.updatedAt).
				Description(tt.args.description).
				Build()

			if tt.err == nil {
				assert.Equal(t, tt.expected, p)
			} else {
				assert.Equal(t, tt.err, err)
			}
		})
	}
}

func TestBuilder_MustBuild(t *testing.T) {
	d := time.Date(1900, 1, 1, 00, 00, 0, 1, time.UTC)
	pid := NewID()
	tid := id.NewWorkspaceID()

	type args struct {
		name, description string
		id                ID
		isArchived        bool
		updatedAt         time.Time
		workspace         WorkspaceID
	}

	tests := []struct {
		name     string
		args     args
		expected *Project
		err      error
	}{
		{
			name: "build normal project",
			args: args{
				name:        "xxx.aaa",
				description: "ddd",
				id:          pid,
				isArchived:  false,
				updatedAt:   d,
				workspace:   tid,
			},
			expected: &Project{
				id:          pid,
				description: "ddd",
				name:        "xxx.aaa",
				isArchived:  false,
				updatedAt:   d,
				workspace:   tid,
			},
		},
		{
			name: "zero updated at",
			args: args{
				id: pid,
			},
			expected: &Project{
				id:        pid,
				updatedAt: pid.Timestamp(),
			},
		},
		{
			name: "failed invalid id",
			err:  ErrInvalidID,
		},
	}

	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			t.Parallel()

			build := func() *Project {
				t.Helper()
				return New().
					ID(tt.args.id).
					UpdatedAt(tt.args.updatedAt).
					Workspace(tt.args.workspace).
					Name(tt.args.name).
					UpdatedAt(tt.args.updatedAt).
					Description(tt.args.description).
					MustBuild()
			}

			if tt.err != nil {
				assert.PanicsWithValue(t, tt.err, func() { _ = build() })
			} else {
				assert.Equal(t, tt.expected, build())
			}
		})
	}
}
