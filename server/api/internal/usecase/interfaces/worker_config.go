package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
)

type WorkerConfig interface {
	FindByID(context.Context, id.WorkerConfigID) (*workerconfig.WorkerConfig, error)
	FindByIDs(context.Context, []id.WorkerConfigID) ([]*workerconfig.WorkerConfig, error)
	Fetch(context.Context) (*workerconfig.WorkerConfig, error)
	Update(
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
	) (*workerconfig.WorkerConfig, error)
	Delete(context.Context) error
}
