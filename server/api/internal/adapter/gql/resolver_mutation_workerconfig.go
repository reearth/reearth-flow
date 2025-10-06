package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
)

func (r *mutationResolver) UpdateWorkerConfig(ctx context.Context, input gqlmodel.UpdateWorkerConfigInput) (*gqlmodel.UpdateWorkerConfigPayload, error) {
	wsID, err := gqlmodel.ToID[workerconfig.WorkspaceID](input.WorkspaceID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).WorkerConfig.Update(ctx, interfaces.UpdateWorkerConfigParam{
		WorkspaceID:           workerconfig.WorkspaceID(wsID),
		BootDiskSizeGB:        input.BootDiskSizeGb,
		BootDiskType:          input.BootDiskType,
		ChannelBufferSize:     input.ChannelBufferSize,
		ComputeCpuMilli:       input.ComputeCPUMilli,
		ComputeMemoryMib:      input.ComputeMemoryMib,
		FeatureFlushThreshold: input.FeatureFlushThreshold,
		ImageURL:              input.ImageURL,
		MachineType:           input.MachineType,
		MaxConcurrency:        input.MaxConcurrency,
	})
	if err != nil {
		return nil, err
	}

	return &gqlmodel.UpdateWorkerConfigPayload{
		WorkerConfig: gqlmodel.ToWorkerConfig(res),
	}, nil
}

func (r *mutationResolver) ResetWorkerConfig(ctx context.Context, input gqlmodel.ResetWorkerConfigInput) (*gqlmodel.ResetWorkerConfigPayload, error) {
	wsID, err := gqlmodel.ToID[workerconfig.WorkspaceID](input.WorkspaceID)
	if err != nil {
		return nil, err
	}

	res, err := usecases(ctx).WorkerConfig.Reset(ctx, interfaces.ResetWorkerConfigParam{
		WorkspaceID: workerconfig.WorkspaceID(wsID),
		Fields:      input.Fields,
	})
	if err != nil {
		return nil, err
	}

	return &gqlmodel.ResetWorkerConfigPayload{
		WorkerConfig: gqlmodel.ToWorkerConfig(res),
	}, nil
}
