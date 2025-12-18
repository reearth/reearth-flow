package interactor

import (
	"context"
	"fmt"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
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

func (i *WorkerConfig) FindByID(ctx context.Context, wid id.WorkerConfigID) (*workerconfig.WorkerConfig, error) {
	if err := i.checkPermission(ctx, rbac.ActionRead); err != nil {
		return nil, err
	}

	return i.repo.FindByID(ctx, wid)
}

func (i *WorkerConfig) FindByIDs(ctx context.Context, ids []id.WorkerConfigID) ([]*workerconfig.WorkerConfig, error) {
	if err := i.checkPermission(ctx, rbac.ActionRead); err != nil {
		return nil, err
	}

	return i.repo.FindByIDs(ctx, ids)
}

func (i *WorkerConfig) Fetch(ctx context.Context) (*workerconfig.WorkerConfig, error) {
	if err := i.checkPermission(ctx, rbac.ActionRead); err != nil {
		return nil, err
	}

	return i.repo.FindAll(ctx)
}

func (i *WorkerConfig) Update(
	ctx context.Context,
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

	cfg, err := i.repo.FindAll(ctx)
	if err != nil {
		return nil, err
	}

	if cfg == nil {
		cfg = workerconfig.New()
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

func (i *WorkerConfig) Delete(ctx context.Context) error {
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

	cfg, err := i.repo.FindAll(ctx)
	if err != nil {
		return err
	}
	if cfg == nil {
		return nil
	}

	if err := i.repo.Remove(ctx, cfg.ID()); err != nil {
		return err
	}

	tx.Commit()
	return nil
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

func getConfigLimits() ConfigLimits {
	return ConfigLimits{
		AllowedMachineTypes: map[string]bool{
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
		},
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
}

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
	limits := getConfigLimits()

	if machineType != nil && *machineType != "" {
		if !limits.AllowedMachineTypes[*machineType] {
			return fmt.Errorf("machine type '%s' is not allowed", *machineType)
		}
	}

	if computeCpuMilli != nil {
		if *computeCpuMilli < 500 {
			return fmt.Errorf("computeCpuMilli must be at least 500, got %d", *computeCpuMilli)
		}
		if *computeCpuMilli > limits.MaxComputeCpuMilli {
			return fmt.Errorf("computeCpuMilli exceeds maximum of %d, got %d", limits.MaxComputeCpuMilli, *computeCpuMilli)
		}
	}

	if computeMemoryMib != nil {
		if *computeMemoryMib < 512 {
			return fmt.Errorf("computeMemoryMib must be at least 512, got %d", *computeMemoryMib)
		}
		if *computeMemoryMib > limits.MaxComputeMemoryMib {
			return fmt.Errorf("computeMemoryMib exceeds maximum of %d, got %d", limits.MaxComputeMemoryMib, *computeMemoryMib)
		}
	}

	if bootDiskSizeGB != nil {
		if *bootDiskSizeGB < 10 {
			return fmt.Errorf("bootDiskSizeGB must be at least 10, got %d", *bootDiskSizeGB)
		}
		if *bootDiskSizeGB > limits.MaxBootDiskSizeGB {
			return fmt.Errorf("bootDiskSizeGB exceeds maximum of %d, got %d", limits.MaxBootDiskSizeGB, *bootDiskSizeGB)
		}
	}

	if taskCount != nil {
		if *taskCount < 1 {
			return fmt.Errorf("taskCount must be at least 1, got %d", *taskCount)
		}
		if *taskCount > limits.MaxTaskCount {
			return fmt.Errorf("taskCount exceeds maximum of %d, got %d", limits.MaxTaskCount, *taskCount)
		}
	}

	if maxConcurrency != nil {
		if *maxConcurrency < 1 {
			return fmt.Errorf("maxConcurrency must be at least 1, got %d", *maxConcurrency)
		}
		if *maxConcurrency > limits.MaxConcurrency {
			return fmt.Errorf("maxConcurrency exceeds maximum of %d, got %d", limits.MaxConcurrency, *maxConcurrency)
		}
	}

	if threadPoolSize != nil {
		if *threadPoolSize < 1 {
			return fmt.Errorf("threadPoolSize must be at least 1, got %d", *threadPoolSize)
		}
		if *threadPoolSize > limits.MaxThreadPoolSize {
			return fmt.Errorf("threadPoolSize exceeds maximum of %d, got %d", limits.MaxThreadPoolSize, *threadPoolSize)
		}
	}

	if channelBufferSize != nil {
		if *channelBufferSize < 1 {
			return fmt.Errorf("channelBufferSize must be at least 1, got %d", *channelBufferSize)
		}
		if *channelBufferSize > limits.MaxChannelBufferSize {
			return fmt.Errorf("channelBufferSize exceeds maximum of %d, got %d", limits.MaxChannelBufferSize, *channelBufferSize)
		}
	}

	if featureFlushThreshold != nil {
		if *featureFlushThreshold < 1 {
			return fmt.Errorf("featureFlushThreshold must be at least 1, got %d", *featureFlushThreshold)
		}
		if *featureFlushThreshold > limits.MaxFeatureFlushThreshold {
			return fmt.Errorf("featureFlushThreshold exceeds maximum of %d, got %d", limits.MaxFeatureFlushThreshold, *featureFlushThreshold)
		}
	}

	if nodeStatusDelayMilli != nil {
		if *nodeStatusDelayMilli < limits.MinNodeStatusDelayMilli {
			return fmt.Errorf("nodeStatusPropagationDelayMilli must be at least %d, got %d", limits.MinNodeStatusDelayMilli, *nodeStatusDelayMilli)
		}
		if *nodeStatusDelayMilli > limits.MaxNodeStatusDelayMilli {
			return fmt.Errorf("nodeStatusPropagationDelayMilli exceeds maximum of %d, got %d", limits.MaxNodeStatusDelayMilli, *nodeStatusDelayMilli)
		}
	}

	return nil
}
