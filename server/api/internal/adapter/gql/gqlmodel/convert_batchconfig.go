package gqlmodel

import (
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/batchconfig"
	"github.com/samber/lo"
)

func ToBatchConfig(config *batchconfig.BatchConfig) *BatchConfig {
	if config == nil {
		return nil
	}

	return &BatchConfig{
		ID:                           ID(config.ID().String()),
		WorkspaceID:                  IDFrom(config.WorkspaceID()),
		CreatedAt:                    config.CreatedAt(),
		UpdatedAt:                    config.UpdatedAt(),
		CreatedBy:                    config.CreatedBy(),
		UpdatedBy:                    config.UpdatedBy(),
		ComputeCPUMilli:              config.ComputeCpuMilli(),
		ComputeMemoryMib:             config.ComputeMemoryMib(),
		BootDiskSizeGb:               config.BootDiskSizeGB(),
		MaxConcurrency:               config.MaxConcurrency(),
		ThreadPoolSize:               config.ThreadPoolSize(),
		ChannelBufferSize:            config.ChannelBufferSize(),
		FeatureFlushThreshold:        config.FeatureFlushThreshold(),
		MachineType:                  config.MachineType(),
		TaskCount:                    config.TaskCount(),
		NodeStatusPropagationDelayMs: config.NodeStatusPropagationDelayMS(),
		BootDiskType:                 config.BootDiskType(),
		ImageURL:                     config.ImageURL(),
		BinaryPath:                   config.BinaryPath(),
		AllowedLocations:             config.AllowedLocations(),
		ChangeHistory:                ToConfigChanges(config.ChangeHistory()),
	}
}

func ToConfigChanges(changes []batchconfig.ConfigChange) []*ConfigChange {
	return lo.Map(changes, func(c batchconfig.ConfigChange, _ int) *ConfigChange {
		return &ConfigChange{
			Timestamp: c.Timestamp,
			ChangedBy: c.ChangedBy,
			FieldName: c.FieldName,
			OldValue:  c.OldValue,
			NewValue:  c.NewValue,
		}
	})
}

func ToEffectiveBatchConfig(effective *interfaces.EffectiveBatchConfig) *EffectiveBatchConfig {
	if effective == nil {
		return nil
	}

	var customConfigID *ID
	if effective.CustomConfigID != nil {
		id := ID(effective.CustomConfigID.String())
		customConfigID = &id
	}

	return &EffectiveBatchConfig{
		WorkspaceID:                  IDFrom(effective.WorkspaceID),
		ComputeCPUMilli:              effective.ComputeCpuMilli,
		ComputeMemoryMib:             effective.ComputeMemoryMib,
		BootDiskSizeGb:               effective.BootDiskSizeGB,
		MaxConcurrency:               effective.MaxConcurrency,
		ThreadPoolSize:               effective.ThreadPoolSize,
		ChannelBufferSize:            effective.ChannelBufferSize,
		FeatureFlushThreshold:        effective.FeatureFlushThreshold,
		MachineType:                  effective.MachineType,
		TaskCount:                    effective.TaskCount,
		NodeStatusPropagationDelayMs: effective.NodeStatusPropagationDelayMS,
		BootDiskType:                 effective.BootDiskType,
		ImageURL:                     effective.ImageURL,
		BinaryPath:                   effective.BinaryPath,
		AllowedLocations:             effective.AllowedLocations,
		HasCustomConfig:              effective.HasCustomConfig,
		CustomConfigID:               customConfigID,
	}
}

func ToBatchConfigValidationErrors(errors []interfaces.BatchConfigValidationError) []*BatchConfigValidationError {
	if errors == nil {
		return nil
	}

	return lo.Map(errors, func(e interfaces.BatchConfigValidationError, _ int) *BatchConfigValidationError {
		return &BatchConfigValidationError{
			Field:   e.Field,
			Message: e.Message,
		}
	})
}

func ToBatchConfigConstraints(constraints *interfaces.BatchConfigConstraints) *BatchConfigConstraints {
	if constraints == nil {
		return nil
	}

	return &BatchConfigConstraints{
		ComputeCPUMilliRange: &IntRange{
			Min: constraints.ComputeCpuMilliMin,
			Max: constraints.ComputeCpuMilliMax,
		},
		ComputeMemoryMibRange: &IntRange{
			Min: constraints.ComputeMemoryMibMin,
			Max: constraints.ComputeMemoryMibMax,
		},
		BootDiskSizeGBRange: &IntRange{
			Min: constraints.BootDiskSizeGBMin,
			Max: constraints.BootDiskSizeGBMax,
		},
		MaxConcurrencyRange: &IntRange{
			Min: constraints.MaxConcurrencyMin,
			Max: constraints.MaxConcurrencyMax,
		},
		ThreadPoolSizeRange: &IntRange{
			Min: constraints.ThreadPoolSizeMin,
			Max: constraints.ThreadPoolSizeMax,
		},
		ChannelBufferSizeRange: &IntRange{
			Min: constraints.ChannelBufferSizeMin,
			Max: constraints.ChannelBufferSizeMax,
		},
		FeatureFlushThresholdRange: &IntRange{
			Min: constraints.FeatureFlushThresholdMin,
			Max: constraints.FeatureFlushThresholdMax,
		},
		TaskCountRange: &IntRange{
			Min: constraints.TaskCountMin,
			Max: constraints.TaskCountMax,
		},
		NodeStatusPropagationDelayMSRange: &IntRange{
			Min: constraints.NodeStatusPropagationDelayMSMin,
			Max: constraints.NodeStatusPropagationDelayMSMax,
		},
		AllowedMachineTypes:  constraints.AllowedMachineTypes,
		AllowedBootDiskTypes: constraints.AllowedBootDiskTypes,
	}
}
