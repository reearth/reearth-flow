package gqlmodel

import (
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearth-flow/api/pkg/workspace"
	"github.com/stretchr/testify/assert"
)

func TestToProject(t *testing.T) {
	pId := project.NewID()
	wsId := workspace.NewID()
	now := time.Now().Truncate(time.Millisecond)

	tests := []struct {
		name string
		args *project.Project
		want *Project
	}{
		{
			name: "nil",
			args: nil,
			want: nil,
		},
		{
			name: "normal",
			args: project.New().
				ID(pId).
				Workspace(wsId).
				Name("aaa").
				Description("bbb").
				UpdatedAt(now).
				MustBuild(),
			want: &Project{
				ID:                IDFrom(pId),
				IsArchived:        false,
				IsBasicAuthActive: false,
				BasicAuthUsername: "",
				BasicAuthPassword: "",
				CreatedAt:         pId.Timestamp(),
				UpdatedAt:         now,
				Name:              "aaa",
				Description:       "bbb",
				WorkspaceID:       IDFrom(wsId),
				Workspace:         nil,
			},
		},
	}
	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			t.Parallel()
			assert.Equal(t, tt.want, ToProject(tt.args))
		})
	}
}
