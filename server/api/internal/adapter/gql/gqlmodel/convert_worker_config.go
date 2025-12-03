package gqlmodel

import (
	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
)

func ToWorkerConfig(cfg *workerconfig.WorkerConfig) *WorkerConfig {
	if cfg == nil {
		return nil
	}

	return &WorkerConfig{
		ID:                              IDFrom(cfg.Workspace()),
		MachineType:                     cfg.MachineType(),
		ComputeCPUMilli:                 cfg.ComputeCpuMilli(),
		ComputeMemoryMib:                cfg.ComputeMemoryMib(),
		BootDiskSizeGb:                  cfg.BootDiskSizeGB(),
		TaskCount:                       cfg.TaskCount(),
		MaxConcurrency:                  cfg.MaxConcurrency(),
		ThreadPoolSize:                  cfg.ThreadPoolSize(),
		ChannelBufferSize:               cfg.ChannelBufferSize(),
		FeatureFlushThreshold:           cfg.FeatureFlushThreshold(),
		NodeStatusPropagationDelayMilli: cfg.NodeStatusPropagationDelayMilli(),
		CreatedAt:                       cfg.CreatedAt(),
		UpdatedAt:                       cfg.UpdatedAt(),
	}
}
