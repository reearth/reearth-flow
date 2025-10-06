package interactor

import (
	"context"
	"errors"
	"strconv"

	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
)

type WorkerConfig struct {
	workerConfigRepo  repo.WorkerConfig
	transaction       usecasex.Transaction
	permissionChecker gateway.PermissionChecker
	config            *config.Config
}

func NewWorkerConfig(
	r *repo.Container,
	permissionChecker gateway.PermissionChecker,
	cfg interface{},
) interfaces.WorkerConfig {
	// Type assert the config
	appConfig, ok := cfg.(*config.Config)
	if !ok {
		appConfig = nil
	}

	return &WorkerConfig{
		workerConfigRepo:  r.WorkerConfig,
		transaction:       r.Transaction,
		permissionChecker: permissionChecker,
		config:            appConfig,
	}
}

func (i *WorkerConfig) checkPermission(ctx context.Context, action string) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceWorkerConfig, action)
}

func (i *WorkerConfig) FindByWorkspace(
	ctx context.Context,
	wsID workerconfig.WorkspaceID,
) (*workerconfig.WorkerConfig, error) {
	if err := i.checkPermission(ctx, rbac.ActionRead); err != nil {
		return nil, err
	}

	wc, err := i.workerConfigRepo.FindByWorkspace(ctx, wsID)
	if err != nil && !errors.Is(err, rerror.ErrNotFound) {
		return nil, err
	}

	return wc, nil
}

func (i *WorkerConfig) GetDefaults(ctx context.Context) (*workerconfig.WorkerConfig, error) {
	if err := i.checkPermission(ctx, rbac.ActionRead); err != nil {
		return nil, err
	}

	// Create a workerconfig with global defaults from environment variables
	bootDiskSizeGB, _ := strconv.Atoi(i.config.Worker_BootDiskSizeGB)
	computeCpuMilli, _ := strconv.Atoi(i.config.Worker_ComputeCpuMilli)
	computeMemoryMib, _ := strconv.Atoi(i.config.Worker_ComputeMemoryMib)
	channelBufferSize, _ := strconv.Atoi(i.config.Worker_ChannelBufferSize)
	featureFlushThreshold, _ := strconv.Atoi(i.config.Worker_FeatureFlushThreshold)
	maxConcurrency, _ := strconv.Atoi(i.config.Worker_MaxConcurrency)

	// Use a nil workspace ID for defaults
	wc := workerconfig.New().
		NewID().
		WorkspaceID(workerconfig.MustWorkspaceID("00000000-0000-0000-0000-000000000000")).
		BootDiskSizeGB(&bootDiskSizeGB).
		BootDiskType(&i.config.Worker_BootDiskType).
		ChannelBufferSize(&channelBufferSize).
		ComputeCpuMilli(&computeCpuMilli).
		ComputeMemoryMib(&computeMemoryMib).
		FeatureFlushThreshold(&featureFlushThreshold).
		ImageURL(&i.config.Worker_ImageURL).
		MachineType(&i.config.Worker_MachineType).
		MaxConcurrency(&maxConcurrency).
		MustBuild()

	return wc, nil
}

func (i *WorkerConfig) Update(
	ctx context.Context,
	param interfaces.UpdateWorkerConfigParam,
) (*workerconfig.WorkerConfig, error) {
	if err := i.checkPermission(ctx, rbac.ActionEdit); err != nil {
		return nil, err
	}

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return nil, err
	}
	defer func() {
		if err2 := tx.End(ctx); err == nil && err2 != nil {
			err = err2
		}
	}()

	ctx = tx.Context()

	// Try to find existing config
	wc, err := i.workerConfigRepo.FindByWorkspace(ctx, param.WorkspaceID)
	if err != nil && !errors.Is(err, rerror.ErrNotFound) {
		return nil, err
	}

	// Create new config if not exists
	if wc == nil {
		wc = workerconfig.New().
			NewID().
			WorkspaceID(param.WorkspaceID).
			MustBuild()
	}

	// Update fields with validation
	if param.BootDiskSizeGB != nil {
		if err := wc.SetBootDiskSizeGB(param.BootDiskSizeGB); err != nil {
			return nil, err
		}
	}

	if param.BootDiskType != nil {
		if err := wc.SetBootDiskType(param.BootDiskType); err != nil {
			return nil, err
		}
	}

	if param.ChannelBufferSize != nil {
		if err := wc.SetChannelBufferSize(param.ChannelBufferSize); err != nil {
			return nil, err
		}
	}

	if param.ComputeCpuMilli != nil {
		if err := wc.SetComputeCpuMilli(param.ComputeCpuMilli); err != nil {
			return nil, err
		}
	}

	if param.ComputeMemoryMib != nil {
		if err := wc.SetComputeMemoryMib(param.ComputeMemoryMib); err != nil {
			return nil, err
		}
	}

	if param.FeatureFlushThreshold != nil {
		if err := wc.SetFeatureFlushThreshold(param.FeatureFlushThreshold); err != nil {
			return nil, err
		}
	}

	if param.ImageURL != nil {
		wc.SetImageURL(param.ImageURL)
	}

	if param.MachineType != nil {
		wc.SetMachineType(param.MachineType)
	}

	if param.MaxConcurrency != nil {
		if err := wc.SetMaxConcurrency(param.MaxConcurrency); err != nil {
			return nil, err
		}
	}

	if err := i.workerConfigRepo.Save(ctx, wc); err != nil {
		return nil, err
	}

	tx.Commit()
	return wc, nil
}

func (i *WorkerConfig) Reset(
	ctx context.Context,
	param interfaces.ResetWorkerConfigParam,
) (*workerconfig.WorkerConfig, error) {
	if err := i.checkPermission(ctx, rbac.ActionEdit); err != nil {
		return nil, err
	}

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return nil, err
	}
	defer func() {
		if err2 := tx.End(ctx); err == nil && err2 != nil {
			err = err2
		}
	}()

	ctx = tx.Context()

	// Find existing config
	wc, err := i.workerConfigRepo.FindByWorkspace(ctx, param.WorkspaceID)
	if err != nil {
		if errors.Is(err, rerror.ErrNotFound) {
			// If no config exists, there's nothing to reset
			return nil, nil
		}
		return nil, err
	}

	// Reset specified fields
	if len(param.Fields) == 0 {
		// Reset all fields
		wc.ResetAll()
	} else {
		for _, field := range param.Fields {
			if err := wc.ResetField(field); err != nil {
				return nil, err
			}
		}
	}

	// If no overrides remain, delete the config
	if !wc.HasOverrides() {
		if err := i.workerConfigRepo.Delete(ctx, wc.ID()); err != nil {
			return nil, err
		}
		tx.Commit()
		return nil, nil
	}

	if err := i.workerConfigRepo.Save(ctx, wc); err != nil {
		return nil, err
	}

	tx.Commit()
	return wc, nil
}
