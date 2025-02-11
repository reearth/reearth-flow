package interactor

import (
	"context"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearth-flow/api/pkg/projectAccess"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/reearth/reearthx/account/accountusecase"
	"github.com/stretchr/testify/assert"
)

func TestProjectAccess_Share(t *testing.T) {
	// prepare
	ctx := context.Background()
	mem := memory.New()
	config := ContainerConfig{
		Host:       "https://example.com",
		SharedPath: "shared",
	}
	i := &ProjectAccess{
		projectRepo:       mem.Project,
		projectAccessRepo: mem.ProjectAccess,
		transaction:       mem.Transaction,
		config:            config,
	}

	// Set up a workspace, project
	ws := workspace.New().NewID().MustBuild()
	_ = mem.Workspace.Save(ctx, ws)

	pid := project.NewID()
	defer project.MockNewID(pid)()
	prj := project.New().ID(pid).Workspace(ws.ID()).Name("testproject").UpdatedAt(time.Now()).MustBuild()
	_ = mem.Project.Save(ctx, prj)

	tests := []struct {
		name        string
		projectID   id.ProjectID
		operator    *usecase.Operator
		wantErr     bool
		wantURLBase string
	}{
		{
			name:      "success: make project public",
			projectID: prj.ID(),
			operator: &usecase.Operator{
				AcOperator: &accountusecase.Operator{
					WritableWorkspaces: workspace.IDList{ws.ID()},
				},
			},
			wantErr:     false,
			wantURLBase: "https://example.com/shared/",
		},
		{
			name:      "failure: project not found",
			projectID: id.ProjectID{},
			operator: &usecase.Operator{
				AcOperator: &accountusecase.Operator{
					WritableWorkspaces: workspace.IDList{ws.ID()},
				},
			},
			wantErr: true,
		},
		{
			name:      "failure: insufficient permissions",
			projectID: prj.ID(),
			operator: &usecase.Operator{
				AcOperator: &accountusecase.Operator{
					ReadableWorkspaces: workspace.IDList{ws.ID()},
				},
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			url, err := i.Share(ctx, tt.projectID, tt.operator)

			if tt.wantErr {
				assert.Error(t, err)
				assert.Empty(t, url)
				return
			}

			assert.NoError(t, err)
			assert.Contains(t, url, tt.wantURLBase)

			pa, err := mem.ProjectAccess.FindByProjectID(ctx, tt.projectID)
			assert.NoError(t, err)
			assert.NotNil(t, pa)
			assert.True(t, pa.IsPublic())
			assert.NotEmpty(t, pa.Token())
		})
	}
}

func TestProjectAccess_Unshare(t *testing.T) {
	// prepare
	ctx := context.Background()
	mem := memory.New()
	config := ContainerConfig{
		Host:       "https://example.com",
		SharedPath: "shared",
	}
	i := &ProjectAccess{
		projectRepo:       mem.Project,
		projectAccessRepo: mem.ProjectAccess,
		transaction:       mem.Transaction,
		config:            config,
	}

	// Set up a workspace, project, and shared project access
	ws := workspace.New().NewID().MustBuild()
	_ = mem.Workspace.Save(ctx, ws)

	pid := project.NewID()
	defer project.MockNewID(pid)()
	prj := project.New().ID(pid).Workspace(ws.ID()).Name("testproject").UpdatedAt(time.Now()).MustBuild()
	_ = mem.Project.Save(ctx, prj)

	pa, _ := projectAccess.New().
		NewID().
		Project(prj.ID()).
		IsPublic(true).
		Token("token").
		Build()
	_ = mem.ProjectAccess.Save(ctx, pa)

	tests := []struct {
		name      string
		projectID id.ProjectID
		operator  *usecase.Operator
		wantErr   bool
	}{
		{
			name:      "success: make project private",
			projectID: prj.ID(),
			operator: &usecase.Operator{
				AcOperator: &accountusecase.Operator{
					WritableWorkspaces: workspace.IDList{ws.ID()},
				},
			},
			wantErr: false,
		},
		{
			name:      "failure: project not found",
			projectID: id.ProjectID{},
			operator: &usecase.Operator{
				AcOperator: &accountusecase.Operator{
					WritableWorkspaces: workspace.IDList{ws.ID()},
				},
			},
			wantErr: true,
		},
		{
			name:      "failure: insufficient permissions",
			projectID: prj.ID(),
			operator: &usecase.Operator{
				AcOperator: &accountusecase.Operator{
					ReadableWorkspaces: workspace.IDList{ws.ID()},
				},
			},
			wantErr: true,
		},
		{
			name:      "failure: project not found",
			projectID: project.NewID(),
			operator: &usecase.Operator{
				AcOperator: &accountusecase.Operator{
					WritableWorkspaces: workspace.IDList{ws.ID()},
				},
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := i.Unshare(ctx, tt.projectID, tt.operator)

			if tt.wantErr {
				assert.Error(t, err)
				return
			}

			assert.NoError(t, err)

			pa, err := mem.ProjectAccess.FindByProjectID(ctx, tt.projectID)
			if err == nil {
				assert.False(t, pa.IsPublic())
				assert.Empty(t, pa.Token())
			}
		})
	}
}
