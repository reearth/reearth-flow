package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
)

type WorkerConfig interface {
	FindByWorkspace(context.Context, workerconfig.WorkspaceID) (*workerconfig.WorkerConfig, error)
	GetDefaults(context.Context) (*workerconfig.WorkerConfig, error)
	Update(context.Context, UpdateWorkerConfigParam) (*workerconfig.WorkerConfig, error)
	Reset(context.Context, ResetWorkerConfigParam) (*workerconfig.WorkerConfig, error)
}

type UpdateWorkerConfigParam struct {
	WorkspaceID           workerconfig.WorkspaceID
	BootDiskSizeGB        *int
	BootDiskType          *string
	ChannelBufferSize     *int
	ComputeCpuMilli       *int
	ComputeMemoryMib      *int
	FeatureFlushThreshold *int
	ImageURL              *string
	MachineType           *string
	MaxConcurrency        *int
}

type ResetWorkerConfigParam struct {
	WorkspaceID workerconfig.WorkspaceID
	Fields      []string
}
