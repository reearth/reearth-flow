package gqlmodel

import (
	"testing"
	"time"

	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestToDeployment(t *testing.T) {
	dID := deployment.NewID()
	headID := deployment.NewID()
	wsID := accountsworkspace.NewID()
	projID := id.NewProjectID()
	now := time.Now().Truncate(time.Millisecond)

	param, err := parameter.New().
		ProjectID(projID).
		Name("area").
		Type(parameter.TypeText).
		Required(true).
		DefaultValue("tokyo").
		Config(map[string]any{"choices": []any{"tokyo", "osaka"}}).
		CreatedAt(now).
		UpdatedAt(now).
		Build()
	require.NoError(t, err)
	wantParam := &Parameter{
		ID:           IDFrom(param.ID()),
		ProjectID:    IDFrom(projID),
		Name:         "area",
		Type:         ParameterTypeText,
		Required:     true,
		DefaultValue: "tokyo",
		Config:       JSON{"choices": []any{"tokyo", "osaka"}},
		CreatedAt:    now,
		UpdatedAt:    now,
	}

	base := func() *deployment.DeploymentBuilder {
		return deployment.New().
			ID(dID).
			Workspace(wsID).
			WorkflowURL("https://example.com/workflow.yml").
			Description("desc").
			Version("v2").
			UpdatedAt(now)
	}

	tests := []struct {
		args *deployment.Deployment
		want *Deployment
		name string
	}{
		{
			name: "nil",
			args: nil,
			want: nil,
		},
		{
			name: "without project, head or parameters",
			args: base().MustBuild(),
			want: &Deployment{
				ID:          IDFrom(dID),
				WorkspaceID: IDFrom(wsID),
				WorkflowURL: "https://example.com/workflow.yml",
				Description: "desc",
				Version:     "v2",
				CreatedAt:   dID.Timestamp(),
				UpdatedAt:   now,
				Parameters:  []*Parameter{},
			},
		},
		{
			name: "with project, head and parameters",
			args: base().
				Project(&projID).
				HeadID(&headID).
				IsHead(true).
				Parameters([]*parameter.Parameter{param}).
				MustBuild(),
			want: &Deployment{
				ID:          IDFrom(dID),
				WorkspaceID: IDFrom(wsID),
				WorkflowURL: "https://example.com/workflow.yml",
				Description: "desc",
				Version:     "v2",
				CreatedAt:   dID.Timestamp(),
				UpdatedAt:   now,
				ProjectID:   IDFromRef(&projID),
				HeadID:      IDFromRef(&headID),
				IsHead:      true,
				Parameters:  []*Parameter{wantParam},
			},
		},
		{
			name: "nil parameters are dropped",
			args: base().Parameters([]*parameter.Parameter{nil, param, nil}).MustBuild(),
			want: &Deployment{
				ID:          IDFrom(dID),
				WorkspaceID: IDFrom(wsID),
				WorkflowURL: "https://example.com/workflow.yml",
				Description: "desc",
				Version:     "v2",
				CreatedAt:   dID.Timestamp(),
				UpdatedAt:   now,
				Parameters:  []*Parameter{wantParam},
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			assert.Equal(t, tt.want, ToDeployment(tt.args))
		})
	}
}

func TestToDeployment_ParametersNeverNil(t *testing.T) {
	// The GraphQL field is a non-null list, so an empty snapshot must not
	// serialize as null.
	for _, params := range [][]*parameter.Parameter{nil, {}, {nil}} {
		d := deployment.New().NewID().Workspace(accountsworkspace.NewID()).Parameters(params).MustBuild()

		got := ToDeployment(d)

		require.NotNil(t, got)
		assert.NotNil(t, got.Parameters)
		assert.Empty(t, got.Parameters)
	}
}
