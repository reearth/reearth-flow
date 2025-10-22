package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/batchconfig"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type WorkerConfigDocument struct {
	Workspace             string    `bson:"workspace"`
	MachineType           *string   `bson:"machine_type,omitempty"`
	ComputeCpuMilli       *int      `bson:"compute_cpu_milli,omitempty"`
	ComputeMemoryMib      *int      `bson:"compute_memory_mib,omitempty"`
	BootDiskSizeGB        *int      `bson:"boot_disk_size_gb,omitempty"`
	TaskCount             *int      `bson:"task_count,omitempty"`
	MaxConcurrency        *int      `bson:"max_concurrency,omitempty"`
	ThreadPoolSize        *int      `bson:"thread_pool_size,omitempty"`
	ChannelBufferSize     *int      `bson:"channel_buffer_size,omitempty"`
	FeatureFlushThreshold *int      `bson:"feature_flush_threshold,omitempty"`
	NodeStatusDelayMilli  *int      `bson:"node_status_delay_ms,omitempty"`
	CreatedAt             time.Time `bson:"created_at"`
	UpdatedAt             time.Time `bson:"updated_at"`
}

func NewWorkerConfig(cfg *batchconfig.WorkerConfig) (*WorkerConfigDocument, string) {
	if cfg == nil {
		return nil, ""
	}

	d := &WorkerConfigDocument{
		Workspace:             cfg.Workspace().String(),
		MachineType:           cfg.MachineType(),
		ComputeCpuMilli:       cfg.ComputeCpuMilli(),
		ComputeMemoryMib:      cfg.ComputeMemoryMib(),
		BootDiskSizeGB:        cfg.BootDiskSizeGB(),
		TaskCount:             cfg.TaskCount(),
		MaxConcurrency:        cfg.MaxConcurrency(),
		ThreadPoolSize:        cfg.ThreadPoolSize(),
		ChannelBufferSize:     cfg.ChannelBufferSize(),
		FeatureFlushThreshold: cfg.FeatureFlushThreshold(),
		NodeStatusDelayMilli:  cfg.NodeStatusPropagationDelayMilli(),
		CreatedAt:             cfg.CreatedAt(),
		UpdatedAt:             cfg.UpdatedAt(),
	}
	return d, cfg.Workspace().String()
}

func (d *WorkerConfigDocument) Model() *batchconfig.WorkerConfig {
	if d == nil {
		return nil
	}
	ws, err := id.WorkspaceIDFrom(d.Workspace)
	if err != nil {
		return nil
	}
	cfg := batchconfig.New(ws)
	cfg.SetMachineType(d.MachineType)
	cfg.SetComputeCpuMilli(d.ComputeCpuMilli)
	cfg.SetComputeMemoryMib(d.ComputeMemoryMib)
	cfg.SetBootDiskSizeGB(d.BootDiskSizeGB)
	cfg.SetTaskCount(d.TaskCount)
	cfg.SetMaxConcurrency(d.MaxConcurrency)
	cfg.SetThreadPoolSize(d.ThreadPoolSize)
	cfg.SetChannelBufferSize(d.ChannelBufferSize)
	cfg.SetFeatureFlushThreshold(d.FeatureFlushThreshold)
	cfg.SetNodeStatusPropagationDelayMilli(d.NodeStatusDelayMilli)
	cfg.ReplaceTimestamps(d.CreatedAt, d.UpdatedAt)
	return cfg
}
