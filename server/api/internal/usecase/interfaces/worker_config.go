package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
)

type WorkerConfig interface {
	FindByWorkspace(context.Context, id.WorkspaceID) (*workerconfig.WorkerConfig, error)
	FindByWorkspaces(context.Context, []id.WorkspaceID) ([]*workerconfig.WorkerConfig, error)
	Update(
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
	) (*workerconfig.WorkerConfig, error)
	Delete(context.Context, id.WorkspaceID) error
}
