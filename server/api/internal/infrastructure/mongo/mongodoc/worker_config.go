package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
)

type WorkerConfigDocument struct {
	ID                    string    `bson:"id"`
	CreatedAt             time.Time `bson:"created_at"`
	UpdatedAt             time.Time `bson:"updated_at"`
	MachineType           *string   `bson:"machine_type,omitempty"`
	ComputeCpuMilli       *int      `bson:"compute_cpu_milli,omitempty"`
	ComputeMemoryMib      *int      `bson:"compute_memory_mib,omitempty"`
	BootDiskSizeGB        *int      `bson:"boot_disk_size_gb,omitempty"`
	TaskCount             *int      `bson:"task_count,omitempty"`
	MaxConcurrency        *int      `bson:"max_concurrency,omitempty"`
	ThreadPoolSize        *int      `bson:"thread_pool_size,omitempty"`
	ChannelBufferSize     *int      `bson:"channel_buffer_size,omitempty"`
	FeatureFlushThreshold *int      `bson:"feature_flush_threshold,omitempty"`
	NodeStatusDelayMilli  *int      `bson:"node_status_delay_milli,omitempty"`
}

func NewWorkerConfig(cfg *workerconfig.WorkerConfig) (*WorkerConfigDocument, string) {
	if cfg == nil {
		return nil, ""
	}

	d := &WorkerConfigDocument{
		ID:                    cfg.ID().String(),
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
	return d, cfg.ID().String()
}

type WorkerConfigConsumer = Consumer[*WorkerConfigDocument, *workerconfig.WorkerConfig]

func NewWorkerConfigConsumer() *WorkerConfigConsumer {
	return NewConsumer[*WorkerConfigDocument](func(a *workerconfig.WorkerConfig) bool {
		return true
	})
}

func (d *WorkerConfigDocument) Model() (*workerconfig.WorkerConfig, error) {
	if d == nil {
		return nil, nil
	}
	wid, err := id.WorkerConfigIDFrom(d.ID)
	if err != nil {
		return nil, err
	}
	cfg, err := workerconfig.NewBuilder().
		ID(wid).
		MachineType(d.MachineType).
		ComputeCpuMilli(d.ComputeCpuMilli).
		ComputeMemoryMib(d.ComputeMemoryMib).
		BootDiskSizeGB(d.BootDiskSizeGB).
		TaskCount(d.TaskCount).
		MaxConcurrency(d.MaxConcurrency).
		ThreadPoolSize(d.ThreadPoolSize).
		ChannelBufferSize(d.ChannelBufferSize).
		FeatureFlushThreshold(d.FeatureFlushThreshold).
		NodeStatusPropagationDelayMilli(d.NodeStatusDelayMilli).
		CreatedAt(d.CreatedAt).
		UpdatedAt(d.UpdatedAt).
		Build()
	if err != nil {
		return nil, err
	}
	return cfg, nil
}
