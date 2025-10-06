package gqlmodel

import (
	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
	"github.com/samber/lo"
)

func ToWorkerConfig(wc *workerconfig.WorkerConfig) *WorkerConfig {
	if wc == nil {
		return nil
	}

	return &WorkerConfig{
		ID:                    IDFrom(wc.ID()),
		WorkspaceID:           IDFrom(wc.WorkspaceID()),
		BootDiskSizeGb:        wc.BootDiskSizeGB(),
		BootDiskType:          wc.BootDiskType(),
		ChannelBufferSize:     wc.ChannelBufferSize(),
		ComputeCPUMilli:       wc.ComputeCpuMilli(),
		ComputeMemoryMib:      wc.ComputeMemoryMib(),
		FeatureFlushThreshold: wc.FeatureFlushThreshold(),
		ImageURL:              wc.ImageURL(),
		MachineType:           wc.MachineType(),
		MaxConcurrency:        wc.MaxConcurrency(),
		CreatedAt:             wc.CreatedAt(),
		UpdatedAt:             wc.UpdatedAt(),
	}
}

func ToWorkerConfigList(wcs []*workerconfig.WorkerConfig) []*WorkerConfig {
	return lo.Map(wcs, func(wc *workerconfig.WorkerConfig, _ int) *WorkerConfig {
		return ToWorkerConfig(wc)
	})
}
