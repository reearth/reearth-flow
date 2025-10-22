package e2e

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/testutil/factory"
	"github.com/reearth/reearth-flow/api/pkg/batchconfig"
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

	e, r, _ := StartServerAndRepos(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{Disabled: true},
	}, true, true, mock)

	ctx := context.Background()

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

	saved, err := r.WorkerConfig.FindByWorkspace(ctx, wID)
	assert.NoError(t, err)
	assert.NotNil(t, saved)
	assert.Equal(t, "e2-standard-16", *saved.MachineType())
	assert.Equal(t, 32000, *saved.ComputeCpuMilli())
	assert.Equal(t, 32768, *saved.ComputeMemoryMib())
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

	e, r, _ := StartServerAndRepos(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{Disabled: true},
	}, true, true, mock)

	ctx := context.Background()

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

	saved, err := r.WorkerConfig.FindByWorkspace(ctx, wID)
	assert.NoError(t, err)
	assert.NotNil(t, saved)
	assert.Equal(t, "e2-standard-8", *saved.MachineType())
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

	e, _, _ := StartServerAndRepos(t, &config.Config{
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

	e, _, _ := StartServerAndRepos(t, &config.Config{
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

func TestUpdateWorkerConfig_PartialUpdate(t *testing.T) {
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

	e, r, _ := StartServerAndRepos(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{Disabled: true},
	}, true, true, mock)

	ctx := context.Background()

	query1 := fmt.Sprintf(`mutation {
		updateWorkerConfig(input: {
			workspaceId: "%s"
			computeCpuMilli: 4000
			computeMemoryMib: 8000
		}) {
			config {
				workspace
				computeCpuMilli
				computeMemoryMib
				machineType
			}
		}
	}`, wID)

	request := GraphQLRequest{
		OperationName: "updateWorkerConfig",
		Query:         query1,
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

	cfg.Value("computeCpuMilli").Number().IsEqual(4000)
	cfg.Value("computeMemoryMib").Number().IsEqual(8000)
	cfg.Value("machineType").IsNull()

	saved, err := r.WorkerConfig.FindByWorkspace(ctx, wID)
	assert.NoError(t, err)
	assert.NotNil(t, saved)
	assert.Equal(t, 4000, *saved.ComputeCpuMilli())
	assert.Equal(t, 8000, *saved.ComputeMemoryMib())
	assert.Nil(t, saved.MachineType())

	query2 := fmt.Sprintf(`mutation {
		updateWorkerConfig(input: {
			workspaceId: "%s"
			machineType: "e2-standard-8"
		}) {
			config {
				workspace
				computeCpuMilli
				computeMemoryMib
				machineType
			}
		}
	}`, wID)

	request2 := GraphQLRequest{
		OperationName: "updateWorkerConfig",
		Query:         query2,
	}
	jsonData2, err := json.Marshal(request2)
	assert.NoError(t, err)

	o2 := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData2).
		Expect().
		Status(http.StatusOK).
		JSON().
		Object()

	cfg2 := o2.Value("data").Object().
		Value("updateWorkerConfig").Object().
		Value("config").Object()

	cfg2.Value("computeCpuMilli").Number().IsEqual(4000)
	cfg2.Value("computeMemoryMib").Number().IsEqual(8000)
	cfg2.Value("machineType").String().IsEqual("e2-standard-8")

	saved2, err := r.WorkerConfig.FindByWorkspace(ctx, wID)
	assert.NoError(t, err)
	assert.NotNil(t, saved2)
	assert.Equal(t, 4000, *saved2.ComputeCpuMilli())
	assert.Equal(t, 8000, *saved2.ComputeMemoryMib())
	assert.Equal(t, "e2-standard-8", *saved2.MachineType())
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

	e, r, _ := StartServerAndRepos(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{Disabled: true},
	}, true, true, mock)

	ctx := context.Background()

	cfg := batchconfig.New(wID)
	cpuMilli := 4000
	cfg.SetComputeCpuMilli(&cpuMilli)
	err := r.WorkerConfig.Save(ctx, cfg)
	assert.NoError(t, err)

	saved, err := r.WorkerConfig.FindByWorkspace(ctx, wID)
	assert.NoError(t, err)
	assert.NotNil(t, saved)

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

	deleted, err := r.WorkerConfig.FindByWorkspace(ctx, wID)
	assert.NoError(t, err)
	assert.Nil(t, deleted)
}

func TestQueryWorkerConfig(t *testing.T) {
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

	e, r, _ := StartServerAndRepos(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{Disabled: true},
	}, true, true, mock)

	ctx := context.Background()

	cfg := batchconfig.New(wID)
	cpuMilli := 8000
	memoryMib := 16384
	machineType := "e2-standard-8"
	cfg.SetComputeCpuMilli(&cpuMilli)
	cfg.SetComputeMemoryMib(&memoryMib)
	cfg.SetMachineType(&machineType)
	err := r.WorkerConfig.Save(ctx, cfg)
	assert.NoError(t, err)

	query := fmt.Sprintf(`query {
		node(id: "%s", type: WORKSPACE) {
			id
			... on Workspace {
				name
				workerConfig {
					workspace
					machineType
					computeCpuMilli
					computeMemoryMib
				}
			}
		}
	}`, wID)

	request := GraphQLRequest{
		OperationName: "node",
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

	workerCfg := o.Value("data").Object().
		Value("node").Object().
		Value("workerConfig").Object()

	workerCfg.Value("workspace").String().IsEqual(wID.String())
	workerCfg.Value("machineType").String().IsEqual("e2-standard-8")
	workerCfg.Value("computeCpuMilli").Number().IsEqual(8000)
	workerCfg.Value("computeMemoryMib").Number().IsEqual(16384)
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

	e, _, _ := StartServerAndRepos(t, &config.Config{
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

func TestUpdateWorkerConfig_AllParameters(t *testing.T) {
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

	e, r, _ := StartServerAndRepos(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{Disabled: true},
	}, true, true, mock)

	ctx := context.Background()

	query := fmt.Sprintf(`mutation {
		updateWorkerConfig(input: {
			workspaceId: "%s"
			machineType: "e2-standard-8"
			computeCpuMilli: 8000
			computeMemoryMib: 16384
			bootDiskSizeGb: 100
			taskCount: 5
			maxConcurrency: 16
			threadPoolSize: 50
			channelBufferSize: 1024
			featureFlushThreshold: 2000
			nodeStatusPropagationDelayMilli: 500
		}) {
			config {
				workspace
				machineType
				computeCpuMilli
				computeMemoryMib
				bootDiskSizeGb
				taskCount
				maxConcurrency
				threadPoolSize
				channelBufferSize
				featureFlushThreshold
				nodeStatusPropagationDelayMilli
				createdAt
				updatedAt
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
	cfg.Value("computeCpuMilli").Number().IsEqual(8000)
	cfg.Value("computeMemoryMib").Number().IsEqual(16384)
	cfg.Value("bootDiskSizeGb").Number().IsEqual(100)
	cfg.Value("taskCount").Number().IsEqual(5)
	cfg.Value("maxConcurrency").Number().IsEqual(16)
	cfg.Value("threadPoolSize").Number().IsEqual(50)
	cfg.Value("channelBufferSize").Number().IsEqual(1024)
	cfg.Value("featureFlushThreshold").Number().IsEqual(2000)
	cfg.Value("nodeStatusPropagationDelayMilli").Number().IsEqual(500)
	cfg.Value("createdAt").NotNull()
	cfg.Value("updatedAt").NotNull()

	saved, err := r.WorkerConfig.FindByWorkspace(ctx, wID)
	assert.NoError(t, err)
	assert.NotNil(t, saved)
	assert.Equal(t, "e2-standard-8", *saved.MachineType())
	assert.Equal(t, 8000, *saved.ComputeCpuMilli())
	assert.Equal(t, 16384, *saved.ComputeMemoryMib())
	assert.Equal(t, 100, *saved.BootDiskSizeGB())
	assert.Equal(t, 5, *saved.TaskCount())
	assert.Equal(t, 16, *saved.MaxConcurrency())
	assert.Equal(t, 50, *saved.ThreadPoolSize())
	assert.Equal(t, 1024, *saved.ChannelBufferSize())
	assert.Equal(t, 2000, *saved.FeatureFlushThreshold())
	assert.Equal(t, 500, *saved.NodeStatusPropagationDelayMilli())
}
