package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
	"github.com/reearth/reearthx/account/accountdomain"
)

type WorkerConfigDocument struct {
	ID                    string    `bson:"_id"`
	WorkspaceID           string    `bson:"workspace_id"`
	BootDiskSizeGB        *int      `bson:"boot_disk_size_gb,omitempty"`
	BootDiskType          *string   `bson:"boot_disk_type,omitempty"`
	ChannelBufferSize     *int      `bson:"channel_buffer_size,omitempty"`
	ComputeCpuMilli       *int      `bson:"compute_cpu_milli,omitempty"`
	ComputeMemoryMib      *int      `bson:"compute_memory_mib,omitempty"`
	FeatureFlushThreshold *int      `bson:"feature_flush_threshold,omitempty"`
	ImageURL              *string   `bson:"image_url,omitempty"`
	MachineType           *string   `bson:"machine_type,omitempty"`
	MaxConcurrency        *int      `bson:"max_concurrency,omitempty"`
	CreatedAt             time.Time `bson:"created_at"`
	UpdatedAt             time.Time `bson:"updated_at"`
}

func NewWorkerConfig(wc *workerconfig.WorkerConfig) *WorkerConfigDocument {
	if wc == nil {
		return nil
	}

	return &WorkerConfigDocument{
		ID:                    wc.ID().String(),
		WorkspaceID:           wc.WorkspaceID().String(),
		BootDiskSizeGB:        wc.BootDiskSizeGB(),
		BootDiskType:          wc.BootDiskType(),
		ChannelBufferSize:     wc.ChannelBufferSize(),
		ComputeCpuMilli:       wc.ComputeCpuMilli(),
		ComputeMemoryMib:      wc.ComputeMemoryMib(),
		FeatureFlushThreshold: wc.FeatureFlushThreshold(),
		ImageURL:              wc.ImageURL(),
		MachineType:           wc.MachineType(),
		MaxConcurrency:        wc.MaxConcurrency(),
		CreatedAt:             wc.CreatedAt(),
		UpdatedAt:             wc.UpdatedAt(),
	}
}

func (d *WorkerConfigDocument) Model() (*workerconfig.WorkerConfig, error) {
	if d == nil {
		return nil, nil
	}

	wcID, err := workerconfig.IDFrom(d.ID)
	if err != nil {
		return nil, err
	}

	wsID, err := accountdomain.WorkspaceIDFrom(d.WorkspaceID)
	if err != nil {
		return nil, err
	}

	return workerconfig.New().
		ID(wcID).
		WorkspaceID(wsID).
		BootDiskSizeGB(d.BootDiskSizeGB).
		BootDiskType(d.BootDiskType).
		ChannelBufferSize(d.ChannelBufferSize).
		ComputeCpuMilli(d.ComputeCpuMilli).
		ComputeMemoryMib(d.ComputeMemoryMib).
		FeatureFlushThreshold(d.FeatureFlushThreshold).
		ImageURL(d.ImageURL).
		MachineType(d.MachineType).
		MaxConcurrency(d.MaxConcurrency).
		CreatedAt(d.CreatedAt).
		UpdatedAt(d.UpdatedAt).
		Build()
}
