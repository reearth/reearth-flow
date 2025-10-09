package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearthx/account/accountdomain"
)

// Query resolvers

func (r *queryResolver) BatchConfig(ctx context.Context, workspaceID gqlmodel.ID) (*gqlmodel.BatchConfig, error) {
	workspaceIDParsed, err := accountdomain.WorkspaceIDFrom(string(workspaceID))
	if err != nil {
		return nil, err
	}

	config, err := usecases(ctx).BatchConfig.GetBatchConfig(ctx, interfaces.GetBatchConfigParam{
		WorkspaceID: workspaceIDParsed,
		Operator:    nil, // Permission check handled in usecase
	})

	if err != nil {
		return nil, err
	}

	if config == nil {
		return nil, nil
	}

	return gqlmodel.ToBatchConfig(config), nil
}

func (r *queryResolver) EffectiveBatchConfig(ctx context.Context, workspaceID gqlmodel.ID) (*gqlmodel.EffectiveBatchConfig, error) {
	workspaceIDParsed, err := accountdomain.WorkspaceIDFrom(string(workspaceID))
	if err != nil {
		return nil, err
	}

	effective, err := usecases(ctx).BatchConfig.GetEffectiveBatchConfig(ctx, interfaces.GetEffectiveBatchConfigParam{
		WorkspaceID: workspaceIDParsed,
		Operator:    nil, // Permission check handled in usecase
	})

	if err != nil {
		return nil, err
	}

	return gqlmodel.ToEffectiveBatchConfig(effective), nil
}

func (r *queryResolver) BatchConfigConstraints(ctx context.Context) (*gqlmodel.BatchConfigConstraints, error) {
	constraints, err := usecases(ctx).BatchConfig.GetBatchConfigConstraints(ctx)
	if err != nil {
		return nil, err
	}

	return gqlmodel.ToBatchConfigConstraints(constraints), nil
}

// Mutation resolvers

func (r *mutationResolver) UpdateBatchConfig(ctx context.Context, input gqlmodel.UpdateBatchConfigInput) (*gqlmodel.UpdateBatchConfigPayload, error) {
	workspaceID, err := accountdomain.WorkspaceIDFrom(string(input.WorkspaceID))
	if err != nil {
		return nil, err
	}

	param := interfaces.UpdateBatchConfigParam{
		WorkspaceID:                  workspaceID,
		Operator:                     nil, // Permission check handled in usecase
		ComputeCpuMilli:              input.ComputeCPUMilli,
		ComputeMemoryMib:             input.ComputeMemoryMib,
		BootDiskSizeGB:               input.BootDiskSizeGb,
		MaxConcurrency:               input.MaxConcurrency,
		ThreadPoolSize:               input.ThreadPoolSize,
		ChannelBufferSize:            input.ChannelBufferSize,
		FeatureFlushThreshold:        input.FeatureFlushThreshold,
		MachineType:                  input.MachineType,
		TaskCount:                    input.TaskCount,
		NodeStatusPropagationDelayMS: input.NodeStatusPropagationDelayMs,
		BootDiskType:                 input.BootDiskType,
		ImageURL:                     input.ImageURL,
		BinaryPath:                   input.BinaryPath,
		AllowedLocations:             input.AllowedLocations,
	}

	config, validationErrors, err := usecases(ctx).BatchConfig.UpdateBatchConfig(ctx, param)
	if err != nil {
		return nil, err
	}

	return &gqlmodel.UpdateBatchConfigPayload{
		Config:           gqlmodel.ToBatchConfig(config),
		ValidationErrors: gqlmodel.ToBatchConfigValidationErrors(validationErrors),
	}, nil
}

func (r *mutationResolver) ResetBatchConfig(ctx context.Context, input gqlmodel.ResetBatchConfigInput) (bool, error) {
	workspaceID, err := accountdomain.WorkspaceIDFrom(string(input.WorkspaceID))
	if err != nil {
		return false, err
	}

	err = usecases(ctx).BatchConfig.ResetBatchConfig(ctx, interfaces.ResetBatchConfigParam{
		WorkspaceID: workspaceID,
		Operator:    nil, // Permission check handled in usecase
	})

	if err != nil {
		return false, err
	}

	return true, nil
}

func (r *mutationResolver) ValidateBatchConfig(ctx context.Context, input gqlmodel.ValidateBatchConfigInput) (*gqlmodel.ValidateBatchConfigPayload, error) {
	workspaceID, err := accountdomain.WorkspaceIDFrom(string(input.WorkspaceID))
	if err != nil {
		return nil, err
	}

	param := interfaces.ValidateBatchConfigParam{
		WorkspaceID:                  workspaceID,
		Operator:                     nil, // Permission check handled in usecase
		ComputeCpuMilli:              input.ComputeCPUMilli,
		ComputeMemoryMib:             input.ComputeMemoryMib,
		BootDiskSizeGB:               input.BootDiskSizeGb,
		MaxConcurrency:               input.MaxConcurrency,
		ThreadPoolSize:               input.ThreadPoolSize,
		ChannelBufferSize:            input.ChannelBufferSize,
		FeatureFlushThreshold:        input.FeatureFlushThreshold,
		MachineType:                  input.MachineType,
		TaskCount:                    input.TaskCount,
		NodeStatusPropagationDelayMS: input.NodeStatusPropagationDelayMs,
		BootDiskType:                 input.BootDiskType,
		ImageURL:                     input.ImageURL,
		BinaryPath:                   input.BinaryPath,
		AllowedLocations:             input.AllowedLocations,
	}

	valid, validationErrors, err := usecases(ctx).BatchConfig.ValidateBatchConfig(ctx, param)
	if err != nil {
		return nil, err
	}

	return &gqlmodel.ValidateBatchConfigPayload{
		Valid:  valid,
		Errors: gqlmodel.ToBatchConfigValidationErrors(validationErrors),
	}, nil
}
