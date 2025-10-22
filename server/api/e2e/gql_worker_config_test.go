package e2e

import (
	"encoding/json"
	"fmt"
	"net/http"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/testutil/factory"
	pkguser "github.com/reearth/reearth-flow/api/pkg/user"
	usermockrepo "github.com/reearth/reearth-flow/api/pkg/user/mockrepo"
	pkgworkspace "github.com/reearth/reearth-flow/api/pkg/workspace"
	workspacemockrepo "github.com/reearth/reearth-flow/api/pkg/workspace/mockrepo"
	"github.com/stretchr/testify/assert"
	"go.uber.org/mock/gomock"
)

func TestUpdateWorkerConfig_Owner(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := pkguser.NewID()
	operator := factory.NewUser(func(b *pkguser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})
	wID := pkgworkspace.NewID()
	ws := factory.NewWorkspace(func(b *pkgworkspace.Builder) {
		b.ID(wID)
		b.Name("test workspace")
		b.Members([]pkgworkspace.Member{
			pkgworkspace.UserMember{
				UserID: operatorID,
				Role:   pkgworkspace.RoleOwner,
			},
		})
	})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockWorkspaceRepo := workspacemockrepo.NewMockWorkspaceRepo(ctrl)
	mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil).AnyTimes()
	mockWorkspaceRepo.EXPECT().FindByID(gomock.Any(), wID).Return(ws, nil).AnyTimes()

	mock := &TestMocks{
		UserRepo:      mockUserRepo,
		WorkspaceRepo: mockWorkspaceRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{Disabled: true},
	}, true, true, mock)

	query := fmt.Sprintf(`mutation {
		updateWorkerConfig(input: {
			workspaceId: "%s"
			machineType: "e2-standard-16"
			computeCpuMilli: 32000
			computeMemoryMib: 32768
			bootDiskSizeGb: 500
			taskCount: 10
			maxConcurrency: 32
		}) {
			config {
				workspace
				machineType
				computeCpuMilli
				computeMemoryMib
				bootDiskSizeGb
				taskCount
				maxConcurrency
			}
		}
	}`, wID)

	request := GraphQLRequest{
		OperationName: "updateWorkerConfig",
		Query:         query,
	}
	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).
		Expect().
		Status(http.StatusOK).
		JSON().
		Object()

	cfg := o.Value("data").Object().
		Value("updateWorkerConfig").Object().
		Value("config").Object()

	cfg.Value("workspace").String().IsEqual(wID.String())
	cfg.Value("machineType").String().IsEqual("e2-standard-16")
	cfg.Value("computeCpuMilli").Number().IsEqual(32000)
	cfg.Value("computeMemoryMib").Number().IsEqual(32768)
	cfg.Value("bootDiskSizeGb").Number().IsEqual(500)
	cfg.Value("taskCount").Number().IsEqual(10)
	cfg.Value("maxConcurrency").Number().IsEqual(32)
}

func TestUpdateWorkerConfig_Maintainer(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := pkguser.NewID()
	operator := factory.NewUser(func(b *pkguser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})
	wID := pkgworkspace.NewID()
	ws := factory.NewWorkspace(func(b *pkgworkspace.Builder) {
		b.ID(wID)
		b.Name("test workspace")
		b.Members([]pkgworkspace.Member{
			pkgworkspace.UserMember{
				UserID: operatorID,
				Role:   pkgworkspace.RoleMaintainer,
			},
		})
	})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockWorkspaceRepo := workspacemockrepo.NewMockWorkspaceRepo(ctrl)
	mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil).AnyTimes()
	mockWorkspaceRepo.EXPECT().FindByID(gomock.Any(), wID).Return(ws, nil).AnyTimes()

	mock := &TestMocks{
		UserRepo:      mockUserRepo,
		WorkspaceRepo: mockWorkspaceRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{Disabled: true},
	}, true, true, mock)

	query := fmt.Sprintf(`mutation {
		updateWorkerConfig(input: {
			workspaceId: "%s"
			machineType: "e2-standard-8"
			computeCpuMilli: 16000
			computeMemoryMib: 32768
		}) {
			config {
				workspace
				machineType
				computeCpuMilli
				computeMemoryMib
			}
		}
	}`, wID)

	request := GraphQLRequest{
		OperationName: "updateWorkerConfig",
		Query:         query,
	}
	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).
		Expect().
		Status(http.StatusOK).
		JSON().
		Object()

	cfg := o.Value("data").Object().
		Value("updateWorkerConfig").Object().
		Value("config").Object()

	cfg.Value("machineType").String().IsEqual("e2-standard-8")
	cfg.Value("computeCpuMilli").Number().IsEqual(16000)
}

func TestUpdateWorkerConfig_Maintainer_ExceedsLimit(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := pkguser.NewID()
	operator := factory.NewUser(func(b *pkguser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})
	wID := pkgworkspace.NewID()
	ws := factory.NewWorkspace(func(b *pkgworkspace.Builder) {
		b.ID(wID)
		b.Name("test workspace")
		b.Members([]pkgworkspace.Member{
			pkgworkspace.UserMember{
				UserID: operatorID,
				Role:   pkgworkspace.RoleMaintainer,
			},
		})
	})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockWorkspaceRepo := workspacemockrepo.NewMockWorkspaceRepo(ctrl)
	mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil).AnyTimes()
	mockWorkspaceRepo.EXPECT().FindByID(gomock.Any(), wID).Return(ws, nil).AnyTimes()

	mock := &TestMocks{
		UserRepo:      mockUserRepo,
		WorkspaceRepo: mockWorkspaceRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{Disabled: true},
	}, true, true, mock)

	query := fmt.Sprintf(`mutation {
		updateWorkerConfig(input: {
			workspaceId: "%s"
			machineType: "e2-standard-16"
		}) {
			config { workspace }
		}
	}`, wID)

	request := GraphQLRequest{
		OperationName: "updateWorkerConfig",
		Query:         query,
	}
	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).
		Expect().
		Status(http.StatusOK).
		JSON().
		Object()

	o.Value("errors").Array().Value(0).Object().
		Value("message").String().Contains("not allowed for role MAINTAINER")
}

func TestUpdateWorkerConfig_Writer_ExceedsLimit(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := pkguser.NewID()
	operator := factory.NewUser(func(b *pkguser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})
	wID := pkgworkspace.NewID()
	ws := factory.NewWorkspace(func(b *pkgworkspace.Builder) {
		b.ID(wID)
		b.Name("test workspace")
		b.Members([]pkgworkspace.Member{
			pkgworkspace.UserMember{
				UserID: operatorID,
				Role:   pkgworkspace.RoleWriter,
			},
		})
	})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockWorkspaceRepo := workspacemockrepo.NewMockWorkspaceRepo(ctrl)
	mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil).AnyTimes()
	mockWorkspaceRepo.EXPECT().FindByID(gomock.Any(), wID).Return(ws, nil).AnyTimes()

	mock := &TestMocks{
		UserRepo:      mockUserRepo,
		WorkspaceRepo: mockWorkspaceRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{Disabled: true},
	}, true, true, mock)

	query := fmt.Sprintf(`mutation {
		updateWorkerConfig(input: {
			workspaceId: "%s"
			computeCpuMilli: 16000
		}) {
			config { workspace }
		}
	}`, wID)

	request := GraphQLRequest{
		OperationName: "updateWorkerConfig",
		Query:         query,
	}
	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).
		Expect().
		Status(http.StatusOK).
		JSON().
		Object()

	o.Value("errors").Array().Value(0).Object().
		Value("message").String().Contains("exceeds maximum of 8000 for role WRITER")
}

func TestDeleteWorkerConfig(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := pkguser.NewID()
	operator := factory.NewUser(func(b *pkguser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})
	wID := pkgworkspace.NewID()
	ws := factory.NewWorkspace(func(b *pkgworkspace.Builder) {
		b.ID(wID)
		b.Name("test workspace")
		b.Members([]pkgworkspace.Member{
			pkgworkspace.UserMember{
				UserID: operatorID,
				Role:   pkgworkspace.RoleOwner,
			},
		})
	})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockWorkspaceRepo := workspacemockrepo.NewMockWorkspaceRepo(ctrl)
	mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil).AnyTimes()
	mockWorkspaceRepo.EXPECT().FindByID(gomock.Any(), wID).Return(ws, nil).AnyTimes()

	mock := &TestMocks{
		UserRepo:      mockUserRepo,
		WorkspaceRepo: mockWorkspaceRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{Disabled: true},
	}, true, true, mock)

	query := fmt.Sprintf(`mutation {
		deleteWorkerConfig(input: {
			workspaceId: "%s"
		}) {
			workspaceId
		}
	}`, wID)

	request := GraphQLRequest{
		OperationName: "deleteWorkerConfig",
		Query:         query,
	}
	jsonData, err := json.Marshal(request)
	assert.NoError(t, err)

	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).
		Expect().
		Status(http.StatusOK).
		JSON().
		Object()

	o.Value("data").Object().
		Value("deleteWorkerConfig").Object().
		Value("workspaceId").String().IsEqual(wID.String())
}

func TestUpdateWorkerConfig_ValidationErrors(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := pkguser.NewID()
	operator := factory.NewUser(func(b *pkguser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})
	wID := pkgworkspace.NewID()
	ws := factory.NewWorkspace(func(b *pkgworkspace.Builder) {
		b.ID(wID)
		b.Name("test workspace")
		b.Members([]pkgworkspace.Member{
			pkgworkspace.UserMember{
				UserID: operatorID,
				Role:   pkgworkspace.RoleOwner,
			},
		})
	})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockWorkspaceRepo := workspacemockrepo.NewMockWorkspaceRepo(ctrl)
	mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil).AnyTimes()
	mockWorkspaceRepo.EXPECT().FindByID(gomock.Any(), wID).Return(ws, nil).AnyTimes()

	mock := &TestMocks{
		UserRepo:      mockUserRepo,
		WorkspaceRepo: mockWorkspaceRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{Disabled: true},
	}, true, true, mock)

	tests := []struct {
		name          string
		param         string
		value         int
		expectedError string
	}{
		{
			name:          "CPU too low",
			param:         "computeCpuMilli",
			value:         100,
			expectedError: "must be at least 500",
		},
		{
			name:          "Memory too low",
			param:         "computeMemoryMib",
			value:         100,
			expectedError: "must be at least 512",
		},
		{
			name:          "Disk too low",
			param:         "bootDiskSizeGb",
			value:         5,
			expectedError: "must be at least 10",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			query := fmt.Sprintf(`mutation {
				updateWorkerConfig(input: {
					workspaceId: "%s"
					%s: %d
				}) {
					config { workspace }
				}
			}`, wID, tt.param, tt.value)

			request := GraphQLRequest{
				OperationName: "updateWorkerConfig",
				Query:         query,
			}
			jsonData, err := json.Marshal(request)
			assert.NoError(t, err)

			o := e.POST("/api/graphql").
				WithHeader("authorization", "Bearer test").
				WithHeader("Content-Type", "application/json").
				WithHeader("X-Reearth-Debug-User", operatorID.String()).
				WithBytes(jsonData).
				Expect().
				Status(http.StatusOK).
				JSON().
				Object()

			o.Value("errors").Array().Value(0).Object().
				Value("message").String().Contains(tt.expectedError)
		})
	}
}
