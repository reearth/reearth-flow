package interactor

import (
	"context"
	"errors"
	"strconv"

	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/batchconfig"
	"github.com/reearth/reearthx/account/accountdomain"
)

var (
	ErrBatchConfigNotFound       = errors.New("batch config not found")
	ErrInvalidWorkspaceID        = errors.New("invalid workspace ID")
	ErrWorkspacePermissionDenied = errors.New("workspace permission denied")
	ErrTierCPermissionRequired   = errors.New("tier C parameter modification requires admin permission")
)

type BatchConfig struct {
	repos     *repo.Container
	config    *config.Config
	validator *batchconfig.Validator
}

func NewBatchConfig(repos *repo.Container, appConfig *config.Config) interfaces.BatchConfig {
	return &BatchConfig{
		repos:     repos,
		config:    appConfig,
		validator: batchconfig.NewValidator(),
	}
}

func (i *BatchConfig) GetBatchConfig(ctx context.Context, param interfaces.GetBatchConfigParam) (*batchconfig.BatchConfig, error) {
	// Check workspace permission
	if err := i.checkWorkspacePermission(ctx, param.WorkspaceID, param.Operator); err != nil {
		return nil, err
	}

	// Fetch custom configuration
	config, err := i.repos.BatchConfig.FindByWorkspaceID(ctx, param.WorkspaceID)
	if err != nil {
		return nil, err
	}

	return config, nil
}

func (i *BatchConfig) GetEffectiveBatchConfig(ctx context.Context, param interfaces.GetEffectiveBatchConfigParam) (*interfaces.EffectiveBatchConfig, error) {
	// Check workspace permission
	if err := i.checkWorkspacePermission(ctx, param.WorkspaceID, param.Operator); err != nil {
		return nil, err
	}

	// Fetch custom configuration
	customConfig, err := i.repos.BatchConfig.FindByWorkspaceID(ctx, param.WorkspaceID)
	if err != nil {
		return nil, err
	}

	// Build effective configuration by merging custom config with environment defaults
	effective := i.buildEffectiveConfig(param.WorkspaceID, customConfig)
	return effective, nil
}

func (i *BatchConfig) UpdateBatchConfig(ctx context.Context, param interfaces.UpdateBatchConfigParam) (*batchconfig.BatchConfig, []interfaces.BatchConfigValidationError, error) {
	// Check workspace permission
	if err := i.checkWorkspacePermission(ctx, param.WorkspaceID, param.Operator); err != nil {
		return nil, nil, err
	}

	// Check if Tier C parameters are being modified and require admin permission
	if i.hasTierCChanges(param) {
		if !i.isWorkspaceAdmin(ctx, param.WorkspaceID, param.Operator) {
			return nil, nil, ErrTierCPermissionRequired
		}
	}

	// Fetch existing config or create new one
	existing, err := i.repos.BatchConfig.FindByWorkspaceID(ctx, param.WorkspaceID)
	if err != nil {
		return nil, nil, err
	}

	var cfg *batchconfig.BatchConfig
	if existing == nil {
		// Create new configuration
		builder := batchconfig.New().
			NewID().
			WorkspaceID(param.WorkspaceID).
			CreatedBy(param.Operator.User.String()).
			UpdatedBy(param.Operator.User.String())

		cfg = i.applyUpdates(builder, param, param.Operator.User.String()).MustBuild()
	} else {
		// Update existing configuration
		cfg = existing
		i.applyUpdatesToExisting(cfg, param, param.Operator.User.String())
	}

	// Validate the configuration
	validationErrors := i.validateConfig(cfg)
	if len(validationErrors) > 0 {
		return nil, validationErrors, nil
	}

	// Save configuration
	if err := i.repos.BatchConfig.Save(ctx, cfg); err != nil {
		return nil, nil, err
	}

	return cfg, nil, nil
}

func (i *BatchConfig) ResetBatchConfig(ctx context.Context, param interfaces.ResetBatchConfigParam) error {
	// Check workspace permission
	if err := i.checkWorkspacePermission(ctx, param.WorkspaceID, param.Operator); err != nil {
		return err
	}

	// Remove custom configuration (revert to defaults)
	return i.repos.BatchConfig.RemoveByWorkspaceID(ctx, param.WorkspaceID)
}

func (i *BatchConfig) ValidateBatchConfig(ctx context.Context, param interfaces.ValidateBatchConfigParam) (bool, []interfaces.BatchConfigValidationError, error) {
	// Build temporary config for validation
	builder := batchconfig.New().
		NewID().
		WorkspaceID(param.WorkspaceID).
		CreatedBy("validator").
		UpdatedBy("validator")

	tempParam := interfaces.UpdateBatchConfigParam{
		WorkspaceID:                  param.WorkspaceID,
		ComputeCpuMilli:              param.ComputeCpuMilli,
		ComputeMemoryMib:             param.ComputeMemoryMib,
		BootDiskSizeGB:               param.BootDiskSizeGB,
		MaxConcurrency:               param.MaxConcurrency,
		ThreadPoolSize:               param.ThreadPoolSize,
		ChannelBufferSize:            param.ChannelBufferSize,
		FeatureFlushThreshold:        param.FeatureFlushThreshold,
		MachineType:                  param.MachineType,
		TaskCount:                    param.TaskCount,
		NodeStatusPropagationDelayMS: param.NodeStatusPropagationDelayMS,
		BootDiskType:                 param.BootDiskType,
		ImageURL:                     param.ImageURL,
		BinaryPath:                   param.BinaryPath,
		AllowedLocations:             param.AllowedLocations,
	}

	cfg := i.applyUpdates(builder, tempParam, "validator").MustBuild()

	// Validate
	validationErrors := i.validateConfig(cfg)
	valid := len(validationErrors) == 0

	return valid, validationErrors, nil
}

func (i *BatchConfig) GetBatchConfigConstraints(ctx context.Context) (*interfaces.BatchConfigConstraints, error) {
	return &interfaces.BatchConfigConstraints{
		ComputeCpuMilliMin:              batchconfig.MinComputeCpuMilli,
		ComputeCpuMilliMax:              batchconfig.MaxComputeCpuMilli,
		ComputeMemoryMibMin:             batchconfig.MinComputeMemoryMib,
		ComputeMemoryMibMax:             batchconfig.MaxComputeMemoryMib,
		BootDiskSizeGBMin:               batchconfig.MinBootDiskSizeGB,
		BootDiskSizeGBMax:               batchconfig.MaxBootDiskSizeGB,
		MaxConcurrencyMin:               batchconfig.MinMaxConcurrency,
		MaxConcurrencyMax:               batchconfig.MaxMaxConcurrency,
		ThreadPoolSizeMin:               batchconfig.MinThreadPoolSize,
		ThreadPoolSizeMax:               batchconfig.MaxThreadPoolSize,
		ChannelBufferSizeMin:            batchconfig.MinChannelBufferSize,
		ChannelBufferSizeMax:            batchconfig.MaxChannelBufferSize,
		FeatureFlushThresholdMin:        batchconfig.MinFeatureFlushThreshold,
		FeatureFlushThresholdMax:        batchconfig.MaxFeatureFlushThreshold,
		TaskCountMin:                    batchconfig.MinTaskCount,
		TaskCountMax:                    batchconfig.MaxTaskCount,
		NodeStatusPropagationDelayMSMin: batchconfig.MinNodeStatusPropagationDelayMS,
		NodeStatusPropagationDelayMSMax: batchconfig.MaxNodeStatusPropagationDelayMS,
		AllowedMachineTypes:             batchconfig.AllowedMachineTypes,
		AllowedBootDiskTypes:            batchconfig.AllowedBootDiskTypes,
	}, nil
}

// Helper methods

func (i *BatchConfig) checkWorkspacePermission(ctx context.Context, workspaceID accountdomain.WorkspaceID, operator *usecase.Operator) error {
	if operator == nil {
		return ErrWorkspacePermissionDenied
	}

	// Check if user has access to the workspace
	workspace, err := i.repos.Workspace.FindByID(ctx, workspaceID)
	if err != nil {
		return err
	}
	if workspace == nil {
		return ErrInvalidWorkspaceID
	}

	// TODO: Add proper permission check using workspace members
	return nil
}

func (i *BatchConfig) isWorkspaceAdmin(ctx context.Context, workspaceID accountdomain.WorkspaceID, operator *usecase.Operator) bool {
	if operator == nil {
		return false
	}

	// TODO: Implement proper admin check
	// For now, return true to allow testing
	return true
}

func (i *BatchConfig) hasTierCChanges(param interfaces.UpdateBatchConfigParam) bool {
	return param.BootDiskType != nil ||
		param.ImageURL != nil ||
		param.BinaryPath != nil ||
		param.AllowedLocations != nil
}

func (i *BatchConfig) applyUpdates(builder *batchconfig.Builder, param interfaces.UpdateBatchConfigParam, userID string) *batchconfig.Builder {
	if param.ComputeCpuMilli != nil {
		builder = builder.ComputeCpuMilli(param.ComputeCpuMilli)
	}
	if param.ComputeMemoryMib != nil {
		builder = builder.ComputeMemoryMib(param.ComputeMemoryMib)
	}
	if param.BootDiskSizeGB != nil {
		builder = builder.BootDiskSizeGB(param.BootDiskSizeGB)
	}
	if param.MaxConcurrency != nil {
		builder = builder.MaxConcurrency(param.MaxConcurrency)
	}
	if param.ThreadPoolSize != nil {
		builder = builder.ThreadPoolSize(param.ThreadPoolSize)
	}
	if param.ChannelBufferSize != nil {
		builder = builder.ChannelBufferSize(param.ChannelBufferSize)
	}
	if param.FeatureFlushThreshold != nil {
		builder = builder.FeatureFlushThreshold(param.FeatureFlushThreshold)
	}
	if param.MachineType != nil {
		builder = builder.MachineType(param.MachineType)
	}
	if param.TaskCount != nil {
		builder = builder.TaskCount(param.TaskCount)
	}
	if param.NodeStatusPropagationDelayMS != nil {
		builder = builder.NodeStatusPropagationDelayMS(param.NodeStatusPropagationDelayMS)
	}
	if param.BootDiskType != nil {
		builder = builder.BootDiskType(param.BootDiskType)
	}
	if param.ImageURL != nil {
		builder = builder.ImageURL(param.ImageURL)
	}
	if param.BinaryPath != nil {
		builder = builder.BinaryPath(param.BinaryPath)
	}
	if param.AllowedLocations != nil {
		builder = builder.AllowedLocations(param.AllowedLocations)
	}
	return builder
}

func (i *BatchConfig) applyUpdatesToExisting(cfg *batchconfig.BatchConfig, param interfaces.UpdateBatchConfigParam, userID string) {
	if param.ComputeCpuMilli != nil {
		cfg.SetComputeCpuMilli(param.ComputeCpuMilli, userID)
	}
	if param.ComputeMemoryMib != nil {
		cfg.SetComputeMemoryMib(param.ComputeMemoryMib, userID)
	}
	if param.BootDiskSizeGB != nil {
		cfg.SetBootDiskSizeGB(param.BootDiskSizeGB, userID)
	}
	if param.MaxConcurrency != nil {
		cfg.SetMaxConcurrency(param.MaxConcurrency, userID)
	}
	if param.ThreadPoolSize != nil {
		cfg.SetThreadPoolSize(param.ThreadPoolSize, userID)
	}
	if param.ChannelBufferSize != nil {
		cfg.SetChannelBufferSize(param.ChannelBufferSize, userID)
	}
	if param.FeatureFlushThreshold != nil {
		cfg.SetFeatureFlushThreshold(param.FeatureFlushThreshold, userID)
	}
	if param.MachineType != nil {
		cfg.SetMachineType(param.MachineType, userID)
	}
	if param.TaskCount != nil {
		cfg.SetTaskCount(param.TaskCount, userID)
	}
	if param.NodeStatusPropagationDelayMS != nil {
		cfg.SetNodeStatusPropagationDelayMS(param.NodeStatusPropagationDelayMS, userID)
	}
	if param.BootDiskType != nil {
		cfg.SetBootDiskType(param.BootDiskType, userID)
	}
	if param.ImageURL != nil {
		cfg.SetImageURL(param.ImageURL, userID)
	}
	if param.BinaryPath != nil {
		cfg.SetBinaryPath(param.BinaryPath, userID)
	}
	if param.AllowedLocations != nil {
		cfg.SetAllowedLocations(param.AllowedLocations, userID)
	}
}

func (i *BatchConfig) validateConfig(cfg *batchconfig.BatchConfig) []interfaces.BatchConfigValidationError {
	var errors []interfaces.BatchConfigValidationError

	if err := i.validator.ValidateFullConfig(cfg); err != nil {
		errors = append(errors, interfaces.BatchConfigValidationError{
			Field:   "general",
			Message: err.Error(),
		})
	}

	return errors
}

func (i *BatchConfig) buildEffectiveConfig(workspaceID accountdomain.WorkspaceID, customConfig *batchconfig.BatchConfig) *interfaces.EffectiveBatchConfig {
	effective := &interfaces.EffectiveBatchConfig{
		WorkspaceID:     workspaceID,
		HasCustomConfig: customConfig != nil,
	}

	if customConfig != nil {
		id := customConfig.ID()
		effective.CustomConfigID = &id
	}

	// Apply custom config or fall back to environment defaults (use safe defaults if config not set)
	effective.ComputeCpuMilli = i.getIntSafe(customConfig, func(c *batchconfig.BatchConfig) *int { return c.ComputeCpuMilli() }, "2000")
	effective.ComputeMemoryMib = i.getIntSafe(customConfig, func(c *batchconfig.BatchConfig) *int { return c.ComputeMemoryMib() }, "2000")
	effective.BootDiskSizeGB = i.getIntSafe(customConfig, func(c *batchconfig.BatchConfig) *int { return c.BootDiskSizeGB() }, "50")
	effective.MaxConcurrency = i.getIntSafe(customConfig, func(c *batchconfig.BatchConfig) *int { return c.MaxConcurrency() }, "4")
	effective.ThreadPoolSize = i.getIntSafe(customConfig, func(c *batchconfig.BatchConfig) *int { return c.ThreadPoolSize() }, "30")
	effective.ChannelBufferSize = i.getIntSafe(customConfig, func(c *batchconfig.BatchConfig) *int { return c.ChannelBufferSize() }, "256")
	effective.FeatureFlushThreshold = i.getIntSafe(customConfig, func(c *batchconfig.BatchConfig) *int { return c.FeatureFlushThreshold() }, "512")
	effective.TaskCount = i.getIntSafe(customConfig, func(c *batchconfig.BatchConfig) *int { return c.TaskCount() }, "1")
	effective.NodeStatusPropagationDelayMS = i.getIntSafe(customConfig, func(c *batchconfig.BatchConfig) *int { return c.NodeStatusPropagationDelayMS() }, "1000")

	effective.MachineType = i.getStringSafe(customConfig, func(c *batchconfig.BatchConfig) *string { return c.MachineType() }, "e2-standard-4")
	effective.BootDiskType = i.getStringSafe(customConfig, func(c *batchconfig.BatchConfig) *string { return c.BootDiskType() }, "pd-balanced")
	effective.ImageURL = i.getStringSafe(customConfig, func(c *batchconfig.BatchConfig) *string { return c.ImageURL() }, "")
	effective.BinaryPath = i.getStringSafe(customConfig, func(c *batchconfig.BatchConfig) *string { return c.BinaryPath() }, "reearth-flow-worker")

	effective.AllowedLocations = i.getStringSliceSafe(customConfig, func(c *batchconfig.BatchConfig) []string { return c.AllowedLocations() }, []string{})

	return effective
}

func (i *BatchConfig) getIntSafe(cfg *batchconfig.BatchConfig, getter func(*batchconfig.BatchConfig) *int, defaultStr string) int {
	if cfg != nil {
		if val := getter(cfg); val != nil {
			return *val
		}
	}
	// Try config first, then fall back to default
	if i.config != nil {
		// This would need proper field mapping - for now use the fallback
	}
	// Parse default from string
	if val, err := strconv.Atoi(defaultStr); err == nil {
		return val
	}
	return 0
}

func (i *BatchConfig) getStringSafe(cfg *batchconfig.BatchConfig, getter func(*batchconfig.BatchConfig) *string, defaultStr string) string {
	if cfg != nil {
		if val := getter(cfg); val != nil {
			return *val
		}
	}
	// Try config first, then fall back to default
	if i.config != nil {
		// This would need proper field mapping - for now use the fallback
	}
	return defaultStr
}

func (i *BatchConfig) getStringSliceSafe(cfg *batchconfig.BatchConfig, getter func(*batchconfig.BatchConfig) []string, defaultSlice []string) []string {
	if cfg != nil {
		if val := getter(cfg); val != nil {
			return val
		}
	}
	// Try config first, then fall back to default
	if i.config != nil {
		// This would need proper field mapping - for now use the fallback
	}
	return defaultSlice
}

func (i *BatchConfig) getInt(cfg *batchconfig.BatchConfig, getter func(*batchconfig.BatchConfig) *int, defaultStr string) int {
	if cfg != nil {
		if val := getter(cfg); val != nil {
			return *val
		}
	}
	// Parse default from string
	if val, err := strconv.Atoi(defaultStr); err == nil {
		return val
	}
	return 0
}

func (i *BatchConfig) getString(cfg *batchconfig.BatchConfig, getter func(*batchconfig.BatchConfig) *string, defaultStr string) string {
	if cfg != nil {
		if val := getter(cfg); val != nil {
			return *val
		}
	}
	return defaultStr
}

func (i *BatchConfig) getStringSlice(cfg *batchconfig.BatchConfig, getter func(*batchconfig.BatchConfig) []string, defaultSlice []string) []string {
	if cfg != nil {
		if val := getter(cfg); val != nil {
			return val
		}
	}
	return defaultSlice
}
