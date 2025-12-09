package interactor

import (
	"context"
	"fmt"

	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
	"github.com/reearth/reearth-flow/api/pkg/workspace"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/usecasex"
)

type WorkerConfig struct {
	repo              repo.WorkerConfig
	workspaceRepo     accountrepo.Workspace
	transaction       usecasex.Transaction
	permissionChecker gateway.PermissionChecker
}

func NewWorkerConfig(r *repo.Container, permissionChecker gateway.PermissionChecker) interfaces.WorkerConfig {
	return &WorkerConfig{
		repo:              r.WorkerConfig,
		workspaceRepo:     r.Workspace,
		transaction:       r.Transaction,
		permissionChecker: permissionChecker,
	}
}

func (i *WorkerConfig) checkPermission(ctx context.Context, action string) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceWorkspace, action)
}

func (i *WorkerConfig) FindByWorkspace(ctx context.Context, workspace id.WorkspaceID) (*workerconfig.WorkerConfig, error) {
	if err := i.checkPermission(ctx, rbac.ActionRead); err != nil {
		return nil, err
	}

	return i.repo.FindByWorkspace(ctx, workspace)
}

func (i *WorkerConfig) FindByWorkspaces(ctx context.Context, workspaces []id.WorkspaceID) ([]*workerconfig.WorkerConfig, error) {
	if err := i.checkPermission(ctx, rbac.ActionRead); err != nil {
		return nil, err
	}

	return i.repo.FindByWorkspaces(ctx, workspaces)
}

func (i *WorkerConfig) Update(
	ctx context.Context,
	workspaceID id.WorkspaceID,
	machineType *string,
	computeCpuMilli *int,
	computeMemoryMib *int,
	bootDiskSizeGB *int,
	taskCount *int,
	maxConcurrency *int,
	threadPoolSize *int,
	channelBufferSize *int,
	featureFlushThreshold *int,
	nodeStatusDelayMilli *int,
) (*workerconfig.WorkerConfig, error) {
	if err := i.checkPermission(ctx, rbac.ActionEdit); err != nil {
		return nil, err
	}

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return nil, err
	}

	ctx = tx.Context()
	defer func() {
		if err2 := tx.End(ctx); err == nil && err2 != nil {
			err = err2
		}
	}()

	userRole, err := i.getUserWorkspaceRole(ctx, workspaceID)
	if err != nil {
		if skipPermissionCheck {
			userRole = workspace.RoleOwner
		} else {
			return nil, fmt.Errorf("failed to get user workspace role: %w", err)
		}
	}

	if err := validateWorkerConfigByRole(
		userRole,
		machineType,
		computeCpuMilli,
		computeMemoryMib,
		bootDiskSizeGB,
		taskCount,
		maxConcurrency,
		threadPoolSize,
		channelBufferSize,
		featureFlushThreshold,
		nodeStatusDelayMilli,
	); err != nil {
		return nil, err
	}

	cfg, err := i.repo.FindByWorkspace(ctx, workspaceID)
	if err != nil {
		return nil, err
	}

	if cfg == nil {
		cfg = workerconfig.New(workspaceID)
	}

	if machineType != nil {
		cfg.SetMachineType(machineType)
	}
	if computeCpuMilli != nil {
		cfg.SetComputeCpuMilli(computeCpuMilli)
	}
	if computeMemoryMib != nil {
		cfg.SetComputeMemoryMib(computeMemoryMib)
	}
	if bootDiskSizeGB != nil {
		cfg.SetBootDiskSizeGB(bootDiskSizeGB)
	}
	if taskCount != nil {
		cfg.SetTaskCount(taskCount)
	}
	if maxConcurrency != nil {
		cfg.SetMaxConcurrency(maxConcurrency)
	}
	if threadPoolSize != nil {
		cfg.SetThreadPoolSize(threadPoolSize)
	}
	if channelBufferSize != nil {
		cfg.SetChannelBufferSize(channelBufferSize)
	}
	if featureFlushThreshold != nil {
		cfg.SetFeatureFlushThreshold(featureFlushThreshold)
	}
	if nodeStatusDelayMilli != nil {
		cfg.SetNodeStatusPropagationDelayMilli(nodeStatusDelayMilli)
	}

	if err := i.repo.Save(ctx, cfg); err != nil {
		return nil, err
	}

	tx.Commit()
	return cfg, nil
}

func (i *WorkerConfig) Delete(ctx context.Context, workspace id.WorkspaceID) error {
	if err := i.checkPermission(ctx, rbac.ActionEdit); err != nil {
		return err
	}

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return err
	}

	ctx = tx.Context()
	defer func() {
		if err2 := tx.End(ctx); err == nil && err2 != nil {
			err = err2
		}
	}()

	if err := i.repo.Remove(ctx, workspace); err != nil {
		return err
	}

	tx.Commit()
	return nil
}

func (i *WorkerConfig) getUserWorkspaceRole(ctx context.Context, workspaceID id.WorkspaceID) (workspace.Role, error) {
	u := adapter.ReearthxUser(ctx)
	if u == nil {
		return "", fmt.Errorf("user not found in context")
	}

	accWorkspaceID := convertToAccountWorkspaceID(workspaceID)

	ws, err := i.workspaceRepo.FindByID(ctx, accWorkspaceID)
	if err != nil {
		return "", err
	}

	if ws.Members() != nil {
		member := ws.Members().User(accountdomain.UserID(u.ID()))
		if member != nil {
			return workspace.Role(member.Role), nil
		}
	}

	return "", fmt.Errorf("user is not a member of this workspace")
}

func convertToAccountWorkspaceID(flowID id.WorkspaceID) accountdomain.WorkspaceID {
	return accountdomain.WorkspaceID(flowID)
}

type ConfigLimits struct {
	AllowedMachineTypes      map[string]bool
	MaxComputeCpuMilli       int
	MaxComputeMemoryMib      int
	MaxBootDiskSizeGB        int
	MaxTaskCount             int
	MaxConcurrency           int
	MaxThreadPoolSize        int
	MaxChannelBufferSize     int
	MaxFeatureFlushThreshold int
	MaxNodeStatusDelayMilli  int
	MinNodeStatusDelayMilli  int
}

func getConfigLimitsByRole(role workspace.Role) ConfigLimits {
	basicMachineTypes := map[string]bool{
		"e2-standard-2": true,
		"e2-standard-4": true,
	}

	vipMachineTypes := map[string]bool{
		"e2-standard-2": true,
		"e2-standard-4": true,
		"e2-standard-8": true,
		"e2-highmem-2":  true,
		"e2-highmem-4":  true,
		"e2-highcpu-2":  true,
		"e2-highcpu-4":  true,
		"n2-standard-2": true,
		"n2-standard-4": true,
	}

	adminMachineTypes := map[string]bool{
		"e2-standard-2":  true,
		"e2-standard-4":  true,
		"e2-standard-8":  true,
		"e2-standard-16": true,
		"e2-highmem-2":   true,
		"e2-highmem-4":   true,
		"e2-highmem-8":   true,
		"e2-highmem-16":  true,
		"e2-highcpu-2":   true,
		"e2-highcpu-4":   true,
		"e2-highcpu-8":   true,
		"e2-highcpu-16":  true,
		"n2-standard-2":  true,
		"n2-standard-4":  true,
		"n2-standard-8":  true,
		"n2-standard-16": true,
	}

	switch role {
	case workspace.RoleOwner:
		return ConfigLimits{
			AllowedMachineTypes:      adminMachineTypes,
			MaxComputeCpuMilli:       64000,
			MaxComputeMemoryMib:      131072,
			MaxBootDiskSizeGB:        1000,
			MaxTaskCount:             20,
			MaxConcurrency:           64,
			MaxThreadPoolSize:        200,
			MaxChannelBufferSize:     8192,
			MaxFeatureFlushThreshold: 20000,
			MaxNodeStatusDelayMilli:  30000,
			MinNodeStatusDelayMilli:  50,
		}
	case workspace.RoleMaintainer:
		return ConfigLimits{
			AllowedMachineTypes:      vipMachineTypes,
			MaxComputeCpuMilli:       32000,
			MaxComputeMemoryMib:      65536,
			MaxBootDiskSizeGB:        500,
			MaxTaskCount:             10,
			MaxConcurrency:           32,
			MaxThreadPoolSize:        100,
			MaxChannelBufferSize:     4096,
			MaxFeatureFlushThreshold: 10000,
			MaxNodeStatusDelayMilli:  10000,
			MinNodeStatusDelayMilli:  100,
		}
	default:
		return ConfigLimits{
			AllowedMachineTypes:      basicMachineTypes,
			MaxComputeCpuMilli:       8000,
			MaxComputeMemoryMib:      16384,
			MaxBootDiskSizeGB:        200,
			MaxTaskCount:             5,
			MaxConcurrency:           16,
			MaxThreadPoolSize:        50,
			MaxChannelBufferSize:     2048,
			MaxFeatureFlushThreshold: 5000,
			MaxNodeStatusDelayMilli:  5000,
			MinNodeStatusDelayMilli:  200,
		}
	}
}

func validateWorkerConfigByRole(
	role workspace.Role,
	machineType *string,
	computeCpuMilli *int,
	computeMemoryMib *int,
	bootDiskSizeGB *int,
	taskCount *int,
	maxConcurrency *int,
	threadPoolSize *int,
	channelBufferSize *int,
	featureFlushThreshold *int,
	nodeStatusDelayMilli *int,
) error {
	limits := getConfigLimitsByRole(role)

	if machineType != nil && *machineType != "" {
		if !limits.AllowedMachineTypes[*machineType] {
			return fmt.Errorf("machine type '%s' is not allowed for role %s", *machineType, role)
		}
	}

	if computeCpuMilli != nil {
		if *computeCpuMilli < 500 {
			return fmt.Errorf("computeCpuMilli must be at least 500, got %d", *computeCpuMilli)
		}
		if *computeCpuMilli > limits.MaxComputeCpuMilli {
			return fmt.Errorf("computeCpuMilli exceeds maximum of %d for role %s, got %d", limits.MaxComputeCpuMilli, role, *computeCpuMilli)
		}
	}

	if computeMemoryMib != nil {
		if *computeMemoryMib < 512 {
			return fmt.Errorf("computeMemoryMib must be at least 512, got %d", *computeMemoryMib)
		}
		if *computeMemoryMib > limits.MaxComputeMemoryMib {
			return fmt.Errorf("computeMemoryMib exceeds maximum of %d for role %s, got %d", limits.MaxComputeMemoryMib, role, *computeMemoryMib)
		}
	}

	if bootDiskSizeGB != nil {
		if *bootDiskSizeGB < 10 {
			return fmt.Errorf("bootDiskSizeGB must be at least 10, got %d", *bootDiskSizeGB)
		}
		if *bootDiskSizeGB > limits.MaxBootDiskSizeGB {
			return fmt.Errorf("bootDiskSizeGB exceeds maximum of %d for role %s, got %d", limits.MaxBootDiskSizeGB, role, *bootDiskSizeGB)
		}
	}

	if taskCount != nil {
		if *taskCount < 1 {
			return fmt.Errorf("taskCount must be at least 1, got %d", *taskCount)
		}
		if *taskCount > limits.MaxTaskCount {
			return fmt.Errorf("taskCount exceeds maximum of %d for role %s, got %d", limits.MaxTaskCount, role, *taskCount)
		}
	}

	if maxConcurrency != nil {
		if *maxConcurrency < 1 {
			return fmt.Errorf("maxConcurrency must be at least 1, got %d", *maxConcurrency)
		}
		if *maxConcurrency > limits.MaxConcurrency {
			return fmt.Errorf("maxConcurrency exceeds maximum of %d for role %s, got %d", limits.MaxConcurrency, role, *maxConcurrency)
		}
	}

	if threadPoolSize != nil {
		if *threadPoolSize < 1 {
			return fmt.Errorf("threadPoolSize must be at least 1, got %d", *threadPoolSize)
		}
		if *threadPoolSize > limits.MaxThreadPoolSize {
			return fmt.Errorf("threadPoolSize exceeds maximum of %d for role %s, got %d", limits.MaxThreadPoolSize, role, *threadPoolSize)
		}
	}

	if channelBufferSize != nil {
		if *channelBufferSize < 1 {
			return fmt.Errorf("channelBufferSize must be at least 1, got %d", *channelBufferSize)
		}
		if *channelBufferSize > limits.MaxChannelBufferSize {
			return fmt.Errorf("channelBufferSize exceeds maximum of %d for role %s, got %d", limits.MaxChannelBufferSize, role, *channelBufferSize)
		}
	}

	if featureFlushThreshold != nil {
		if *featureFlushThreshold < 1 {
			return fmt.Errorf("featureFlushThreshold must be at least 1, got %d", *featureFlushThreshold)
		}
		if *featureFlushThreshold > limits.MaxFeatureFlushThreshold {
			return fmt.Errorf("featureFlushThreshold exceeds maximum of %d for role %s, got %d", limits.MaxFeatureFlushThreshold, role, *featureFlushThreshold)
		}
	}

	if nodeStatusDelayMilli != nil {
		if *nodeStatusDelayMilli < limits.MinNodeStatusDelayMilli {
			return fmt.Errorf("nodeStatusPropagationDelayMilli must be at least %d for role %s, got %d", limits.MinNodeStatusDelayMilli, role, *nodeStatusDelayMilli)
		}
		if *nodeStatusDelayMilli > limits.MaxNodeStatusDelayMilli {
			return fmt.Errorf("nodeStatusPropagationDelayMilli exceeds maximum of %d for role %s, got %d", limits.MaxNodeStatusDelayMilli, role, *nodeStatusDelayMilli)
		}
	}

	return nil
}
