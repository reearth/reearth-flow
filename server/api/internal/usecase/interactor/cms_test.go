package interactor

import (
	"context"
	"fmt"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/cms"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/reearth/reearthx/account/accountusecase"
	"github.com/reearth/reearthx/appx"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

type mockCMSGateway struct {
	mock.Mock
}

func (m *mockCMSGateway) GetProject(ctx context.Context, projectIDOrAlias string) (*cms.Project, error) {
	args := m.Called(ctx, projectIDOrAlias)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cms.Project), args.Error(1)
}

func (m *mockCMSGateway) ListProjects(ctx context.Context, input cms.ListProjectsInput) ([]*cms.Project, int32, error) {
	args := m.Called(ctx, input)
	if args.Get(0) == nil {
		return nil, args.Get(1).(int32), args.Error(2)
	}
	return args.Get(0).([]*cms.Project), args.Get(1).(int32), args.Error(2)
}

func (m *mockCMSGateway) CreateProject(ctx context.Context, input cms.CreateProjectInput) (*cms.Project, error) {
	args := m.Called(ctx, input)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cms.Project), args.Error(1)
}

func (m *mockCMSGateway) UpdateProject(ctx context.Context, input cms.UpdateProjectInput) (*cms.Project, error) {
	args := m.Called(ctx, input)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cms.Project), args.Error(1)
}

func (m *mockCMSGateway) DeleteProject(ctx context.Context, input cms.DeleteProjectInput) (*cms.DeleteProjectOutput, error) {
	args := m.Called(ctx, input)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cms.DeleteProjectOutput), args.Error(1)
}

func (m *mockCMSGateway) CheckAliasAvailability(ctx context.Context, input cms.CheckAliasAvailabilityInput) (*cms.CheckAliasAvailabilityOutput, error) {
	args := m.Called(ctx, input)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cms.CheckAliasAvailabilityOutput), args.Error(1)
}

func (m *mockCMSGateway) ListModels(ctx context.Context, input cms.ListModelsInput) ([]*cms.Model, int32, error) {
	args := m.Called(ctx, input)
	if args.Get(0) == nil {
		return nil, args.Get(1).(int32), args.Error(2)
	}
	return args.Get(0).([]*cms.Model), args.Get(1).(int32), args.Error(2)
}

func (m *mockCMSGateway) ListItems(ctx context.Context, input cms.ListItemsInput) (*cms.ListItemsOutput, error) {
	args := m.Called(ctx, input)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cms.ListItemsOutput), args.Error(1)
}

func (m *mockCMSGateway) GetModelGeoJSONExportURL(ctx context.Context, input cms.ExportInput) (*cms.ExportOutput, error) {
	args := m.Called(ctx, input)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*cms.ExportOutput), args.Error(1)
}

func TestCMSInteractor_GetCMSProject(t *testing.T) {
	tests := []struct {
		name             string
		projectIDOrAlias string
		mockProject      *cms.Project
		mockError        error
		expectedProject  *cms.Project
		expectedError    string
		setupOperator    bool
	}{
		{
			name:             "successful get project",
			projectIDOrAlias: "project-123",
			mockProject: &cms.Project{
				ID:          "project-123",
				Name:        "Test Project",
				WorkspaceID: "workspace-123",
			},
			mockError: nil,
			expectedProject: &cms.Project{
				ID:          "project-123",
				Name:        "Test Project",
				WorkspaceID: "workspace-123",
			},
			setupOperator: true,
		},
		{
			name:             "no operator",
			projectIDOrAlias: "project-123",
			setupOperator:    false,
			expectedError:    "operator not found",
		},
		{
			name:             "cms gateway error",
			projectIDOrAlias: "project-123",
			mockError:        fmt.Errorf("cms error"),
			expectedError:    "failed to get CMS project: cms error",
			setupOperator:    true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mockCMS := &mockCMSGateway{}
			mockPermChecker := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
				return true, nil
			})

			ctx := context.Background()
			if tt.setupOperator {
				ctx = createContextWithOperator(ctx)
			}

			if tt.setupOperator {
				mockCMS.On("GetProject", ctx, tt.projectIDOrAlias).Return(tt.mockProject, tt.mockError)
			}

			interactor := &cmsInteractor{
				gateways: &gateway.Container{
					CMS: mockCMS,
				},
				permissionChecker: mockPermChecker,
			}

			result, err := interactor.GetCMSProject(ctx, tt.projectIDOrAlias)

			if tt.expectedError != "" {
				assert.Error(t, err)
				assert.Contains(t, err.Error(), tt.expectedError)
				assert.Nil(t, result)
			} else {
				assert.NoError(t, err)
				assert.Equal(t, tt.expectedProject, result)
			}

			mockCMS.AssertExpectations(t)
		})
	}
}

func TestCMSInteractor_CreateCMSProject(t *testing.T) {
	tests := []struct {
		name              string
		input             cms.CreateProjectInput
		mockProject       *cms.Project
		mockError         error
		permissionAllowed bool
		permissionError   error
		expectedProject   *cms.Project
		expectedError     string
		setupOperator     bool
	}{
		{
			name: "successful create project",
			input: cms.CreateProjectInput{
				WorkspaceID: "workspace-123",
				Name:        "New Project",
				Alias:       "new-project",
				Visibility:  cms.VisibilityPublic,
			},
			mockProject: &cms.Project{
				ID:          "project-123",
				Name:        "New Project",
				WorkspaceID: "workspace-123",
			},
			permissionAllowed: true,
			expectedProject: &cms.Project{
				ID:          "project-123",
				Name:        "New Project",
				WorkspaceID: "workspace-123",
			},
			setupOperator: true,
		},
		{
			name: "no operator",
			input: cms.CreateProjectInput{
				WorkspaceID: "workspace-123",
				Name:        "New Project",
			},
			setupOperator: false,
			expectedError: "operator not found",
		},
		{
			name: "permission denied",
			input: cms.CreateProjectInput{
				WorkspaceID: "workspace-123",
				Name:        "New Project",
			},
			permissionAllowed: false,
			expectedError:     "permission denied: cannot create project in workspace workspace-123",
			setupOperator:     true,
		},
		{
			name: "permission check error",
			input: cms.CreateProjectInput{
				WorkspaceID: "workspace-123",
				Name:        "New Project",
			},
			permissionError: fmt.Errorf("permission error"),
			expectedError:   "failed to check permission: permission error",
			setupOperator:   true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mockCMS := &mockCMSGateway{}
			mockPermChecker := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
				if tt.permissionError != nil {
					return false, tt.permissionError
				}
				return tt.permissionAllowed, nil
			})

			ctx := context.Background()
			if tt.setupOperator {
				ctx = createContextWithOperator(ctx)
			}

			if tt.setupOperator && tt.permissionAllowed && tt.permissionError == nil {
				mockCMS.On("CreateProject", ctx, tt.input).Return(tt.mockProject, tt.mockError)
			}

			interactor := &cmsInteractor{
				gateways: &gateway.Container{
					CMS: mockCMS,
				},
				permissionChecker: mockPermChecker,
			}

			result, err := interactor.CreateCMSProject(ctx, tt.input)

			if tt.expectedError != "" {
				assert.Error(t, err)
				assert.Contains(t, err.Error(), tt.expectedError)
				assert.Nil(t, result)
			} else {
				assert.NoError(t, err)
				assert.Equal(t, tt.expectedProject, result)
			}

			mockCMS.AssertExpectations(t)
		})
	}
}

func TestCMSInteractor_UpdateCMSProject(t *testing.T) {
	tests := []struct {
		name              string
		input             cms.UpdateProjectInput
		existingProject   *cms.Project
		updatedProject    *cms.Project
		permissionAllowed bool
		expectedProject   *cms.Project
		expectedError     string
		setupOperator     bool
	}{
		{
			name: "successful update project",
			input: cms.UpdateProjectInput{
				ProjectID: "project-123",
				Name:      stringPtr("Updated Project"),
			},
			existingProject: &cms.Project{
				ID:          "project-123",
				Name:        "Original Project",
				WorkspaceID: "workspace-123",
			},
			updatedProject: &cms.Project{
				ID:          "project-123",
				Name:        "Updated Project",
				WorkspaceID: "workspace-123",
			},
			permissionAllowed: true,
			expectedProject: &cms.Project{
				ID:          "project-123",
				Name:        "Updated Project",
				WorkspaceID: "workspace-123",
			},
			setupOperator: true,
		},
		{
			name: "no operator",
			input: cms.UpdateProjectInput{
				ProjectID: "project-123",
			},
			setupOperator: false,
			expectedError: "operator not found",
		},
		{
			name: "permission denied",
			input: cms.UpdateProjectInput{
				ProjectID: "project-123",
			},
			existingProject: &cms.Project{
				ID:          "project-123",
				WorkspaceID: "workspace-123",
			},
			permissionAllowed: false,
			expectedError:     "permission denied: cannot update project in workspace workspace-123",
			setupOperator:     true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mockCMS := &mockCMSGateway{}
			mockPermChecker := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
				return tt.permissionAllowed, nil
			})

			ctx := context.Background()
			if tt.setupOperator {
				ctx = createContextWithOperator(ctx)
			}

			if tt.setupOperator {
				mockCMS.On("GetProject", ctx, tt.input.ProjectID).Return(tt.existingProject, nil)

				if tt.existingProject != nil && tt.permissionAllowed {
					mockCMS.On("UpdateProject", ctx, tt.input).Return(tt.updatedProject, nil)
				}
			}

			interactor := &cmsInteractor{
				gateways: &gateway.Container{
					CMS: mockCMS,
				},
				permissionChecker: mockPermChecker,
			}

			result, err := interactor.UpdateCMSProject(ctx, tt.input)

			if tt.expectedError != "" {
				assert.Error(t, err)
				assert.Contains(t, err.Error(), tt.expectedError)
				assert.Nil(t, result)
			} else {
				assert.NoError(t, err)
				assert.Equal(t, tt.expectedProject, result)
			}

			mockCMS.AssertExpectations(t)
		})
	}
}

func TestCMSInteractor_DeleteCMSProject(t *testing.T) {
	tests := []struct {
		name              string
		input             cms.DeleteProjectInput
		existingProject   *cms.Project
		deleteOutput      *cms.DeleteProjectOutput
		permissionAllowed bool
		expectedOutput    *cms.DeleteProjectOutput
		expectedError     string
		setupOperator     bool
	}{
		{
			name: "successful delete project",
			input: cms.DeleteProjectInput{
				ProjectID: "project-123",
			},
			existingProject: &cms.Project{
				ID:          "project-123",
				WorkspaceID: "workspace-123",
			},
			deleteOutput: &cms.DeleteProjectOutput{
				ProjectID: "project-123",
			},
			permissionAllowed: true,
			expectedOutput: &cms.DeleteProjectOutput{
				ProjectID: "project-123",
			},
			setupOperator: true,
		},
		{
			name: "no operator",
			input: cms.DeleteProjectInput{
				ProjectID: "project-123",
			},
			setupOperator: false,
			expectedError: "operator not found",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mockCMS := &mockCMSGateway{}
			mockPermChecker := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
				return tt.permissionAllowed, nil
			})

			ctx := context.Background()
			if tt.setupOperator {
				ctx = createContextWithOperator(ctx)
			}

			if tt.setupOperator {
				mockCMS.On("GetProject", ctx, tt.input.ProjectID).Return(tt.existingProject, nil)

				if tt.existingProject != nil && tt.permissionAllowed {
					mockCMS.On("DeleteProject", ctx, tt.input).Return(tt.deleteOutput, nil)
				}
			}

			interactor := &cmsInteractor{
				gateways: &gateway.Container{
					CMS: mockCMS,
				},
				permissionChecker: mockPermChecker,
			}

			result, err := interactor.DeleteCMSProject(ctx, tt.input)

			if tt.expectedError != "" {
				assert.Error(t, err)
				assert.Contains(t, err.Error(), tt.expectedError)
				assert.Nil(t, result)
			} else {
				assert.NoError(t, err)
				assert.Equal(t, tt.expectedOutput, result)
			}

			mockCMS.AssertExpectations(t)
		})
	}
}

func TestCMSInteractor_CheckCMSAliasAvailability(t *testing.T) {
	tests := []struct {
		name           string
		input          cms.CheckAliasAvailabilityInput
		mockOutput     *cms.CheckAliasAvailabilityOutput
		mockError      error
		expectedOutput *cms.CheckAliasAvailabilityOutput
		expectedError  string
		setupOperator  bool
	}{
		{
			name: "alias available",
			input: cms.CheckAliasAvailabilityInput{
				Alias: "test-alias",
			},
			mockOutput: &cms.CheckAliasAvailabilityOutput{
				Available: true,
			},
			expectedOutput: &cms.CheckAliasAvailabilityOutput{
				Available: true,
			},
			setupOperator: true,
		},
		{
			name: "no operator",
			input: cms.CheckAliasAvailabilityInput{
				Alias: "test-alias",
			},
			setupOperator: false,
			expectedError: "operator not found",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mockCMS := &mockCMSGateway{}
			mockPermChecker := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
				return true, nil
			})

			ctx := context.Background()
			if tt.setupOperator {
				ctx = createContextWithOperator(ctx)
			}

			// Setup mock expectations
			if tt.setupOperator {
				mockCMS.On("CheckAliasAvailability", ctx, tt.input).Return(tt.mockOutput, tt.mockError)
			}

			interactor := &cmsInteractor{
				gateways: &gateway.Container{
					CMS: mockCMS,
				},
				permissionChecker: mockPermChecker,
			}

			result, err := interactor.CheckCMSAliasAvailability(ctx, tt.input)

			if tt.expectedError != "" {
				assert.Error(t, err)
				assert.Contains(t, err.Error(), tt.expectedError)
				assert.Nil(t, result)
			} else {
				assert.NoError(t, err)
				assert.Equal(t, tt.expectedOutput, result)
			}

			mockCMS.AssertExpectations(t)
		})
	}
}

func TestCMSInteractor_ListCMSProjects(t *testing.T) {
	tests := []struct {
		name             string
		workspaceID      string
		publicOnly       bool
		mockProjects     []*cms.Project
		mockTotalCount   int32
		mockError        error
		expectedProjects []*cms.Project
		expectedCount    int32
		expectedError    string
		setupOperator    bool
	}{
		{
			name:        "successful list projects",
			workspaceID: "workspace-123",
			publicOnly:  false,
			mockProjects: []*cms.Project{
				{ID: "project-1", Name: "Project 1"},
				{ID: "project-2", Name: "Project 2"},
			},
			mockTotalCount: 2,
			expectedProjects: []*cms.Project{
				{ID: "project-1", Name: "Project 1"},
				{ID: "project-2", Name: "Project 2"},
			},
			expectedCount: 2,
			setupOperator: true,
		},
		{
			name:          "no operator",
			workspaceID:   "workspace-123",
			setupOperator: false,
			expectedError: "operator not found",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mockCMS := &mockCMSGateway{}
			mockPermChecker := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
				return true, nil
			})

			ctx := context.Background()
			if tt.setupOperator {
				ctx = createContextWithOperator(ctx)
			}

			if tt.setupOperator {
				expectedInput := cms.ListProjectsInput{
					WorkspaceID: tt.workspaceID,
					PublicOnly:  tt.publicOnly,
				}
				mockCMS.On("ListProjects", ctx, expectedInput).Return(tt.mockProjects, tt.mockTotalCount, tt.mockError)
			}

			interactor := &cmsInteractor{
				gateways: &gateway.Container{
					CMS: mockCMS,
				},
				permissionChecker: mockPermChecker,
			}

			projects, count, err := interactor.ListCMSProjects(ctx, tt.workspaceID, tt.publicOnly)

			if tt.expectedError != "" {
				assert.Error(t, err)
				assert.Contains(t, err.Error(), tt.expectedError)
				assert.Nil(t, projects)
				assert.Zero(t, count)
			} else {
				assert.NoError(t, err)
				assert.Equal(t, tt.expectedProjects, projects)
				assert.Equal(t, tt.expectedCount, count)
			}

			mockCMS.AssertExpectations(t)
		})
	}
}

func createContextWithOperator(ctx context.Context) context.Context {
	mockAuthInfo := &appx.AuthInfo{
		Token: "test-token",
		Sub:   "user-123",
	}
	mockUser := user.New().NewID().Name("test user").Email("test@example.com").MustBuild()
	mockWorkspace := workspace.New().NewID().Name("test workspace").MustBuild()

	uid := mockUser.ID()
	wsID := mockWorkspace.ID()

	operator := &usecase.Operator{
		AcOperator: &accountusecase.Operator{
			User:                   &uid,
			ReadableWorkspaces:     []workspace.ID{wsID},
			WritableWorkspaces:     []workspace.ID{wsID},
			MaintainableWorkspaces: []workspace.ID{wsID},
			OwningWorkspaces:       []workspace.ID{wsID},
		},
	}

	ctx = adapter.AttachAuthInfo(ctx, mockAuthInfo)
	ctx = adapter.AttachUser(ctx, mockUser)
	ctx = adapter.AttachOperator(ctx, operator)

	return ctx
}

func stringPtr(s string) *string {
	return &s
}
