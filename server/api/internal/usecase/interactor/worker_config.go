package interactor

import (
	"context"
	"fmt"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/batchconfig"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/usecasex"
)

type WorkerConfig struct {
	repo              repo.WorkerConfig
	transaction       usecasex.Transaction
	permissionChecker gateway.PermissionChecker
}

func NewWorkerConfig(r *repo.Container, permissionChecker gateway.PermissionChecker) interfaces.WorkerConfig {
	return &WorkerConfig{
		repo:              r.WorkerConfig,
		transaction:       r.Transaction,
		permissionChecker: permissionChecker,
	}
}

func (i *WorkerConfig) checkPermission(ctx context.Context, action string) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceWorkspace, action)
}

func (i *WorkerConfig) FindByWorkspace(ctx context.Context, workspace id.WorkspaceID) (*batchconfig.WorkerConfig, error) {
	if err := i.checkPermission(ctx, rbac.ActionRead); err != nil {
		return nil, err
	}

	return i.repo.FindByWorkspace(ctx, workspace)
}

func (i *WorkerConfig) Update(
	ctx context.Context,
	workspace id.WorkspaceID,
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
) (*batchconfig.WorkerConfig, error) {
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

	// Note: Workspace existence is validated by permission check

	// Validate inputs
	if err := validateWorkerConfig(
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

	// Get existing config or create new one
	cfg, err := i.repo.FindByWorkspace(ctx, workspace)
	if err != nil {
		return nil, err
	}

	if cfg == nil {
		cfg = batchconfig.New(workspace)
	}

	// Apply updates
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

	// Note: Workspace existence is validated by permission check

	if err := i.repo.Remove(ctx, workspace); err != nil {
		return err
	}

	tx.Commit()
	return nil
}

// validateWorkerConfig validates worker configuration parameters with tier-based limits
func validateWorkerConfig(
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
	// Validate machineType if provided
	if machineType != nil && *machineType != "" {
		validMachineTypes := map[string]bool{
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
		if !validMachineTypes[*machineType] {
			return fmt.Errorf("invalid machine type: %s", *machineType)
		}
	}

	// Validate computeCpuMilli (Tier A: 500-32000)
	if computeCpuMilli != nil {
		if *computeCpuMilli < 500 || *computeCpuMilli > 32000 {
			return fmt.Errorf("computeCpuMilli must be between 500 and 32000, got %d", *computeCpuMilli)
		}
	}

	// Validate computeMemoryMib (Tier A: 512-65536)
	if computeMemoryMib != nil {
		if *computeMemoryMib < 512 || *computeMemoryMib > 65536 {
			return fmt.Errorf("computeMemoryMib must be between 512 and 65536, got %d", *computeMemoryMib)
		}
	}

	// Validate bootDiskSizeGB (Tier A: 10-500)
	if bootDiskSizeGB != nil {
		if *bootDiskSizeGB < 10 || *bootDiskSizeGB > 500 {
			return fmt.Errorf("bootDiskSizeGB must be between 10 and 500, got %d", *bootDiskSizeGB)
		}
	}

	// Validate taskCount (Tier B: 1-10)
	if taskCount != nil {
		if *taskCount < 1 || *taskCount > 10 {
			return fmt.Errorf("taskCount must be between 1 and 10, got %d", *taskCount)
		}
	}

	// Validate maxConcurrency (Tier A: 1-32)
	if maxConcurrency != nil {
		if *maxConcurrency < 1 || *maxConcurrency > 32 {
			return fmt.Errorf("maxConcurrency must be between 1 and 32, got %d", *maxConcurrency)
		}
	}

	// Validate threadPoolSize (Tier A: 1-100)
	if threadPoolSize != nil {
		if *threadPoolSize < 1 || *threadPoolSize > 100 {
			return fmt.Errorf("threadPoolSize must be between 1 and 100, got %d", *threadPoolSize)
		}
	}

	// Validate channelBufferSize (Tier A: 1-4096)
	if channelBufferSize != nil {
		if *channelBufferSize < 1 || *channelBufferSize > 4096 {
			return fmt.Errorf("channelBufferSize must be between 1 and 4096, got %d", *channelBufferSize)
		}
	}

	// Validate featureFlushThreshold (Tier A: 1-10000)
	if featureFlushThreshold != nil {
		if *featureFlushThreshold < 1 || *featureFlushThreshold > 10000 {
			return fmt.Errorf("featureFlushThreshold must be between 1 and 10000, got %d", *featureFlushThreshold)
		}
	}

	// Validate nodeStatusDelayMilli (Tier B: 100-10000)
	if nodeStatusDelayMilli != nil {
		if *nodeStatusDelayMilli < 100 || *nodeStatusDelayMilli > 10000 {
			return fmt.Errorf("nodeStatusPropagationDelayMilli must be between 100 and 10000, got %d", *nodeStatusDelayMilli)
		}
	}

	return nil
}
