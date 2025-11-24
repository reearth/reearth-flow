package workerconfig

import (
	"errors"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
)

type Builder struct {
	c *WorkerConfig
}

func NewBuilder() *Builder {
	return &Builder{c: &WorkerConfig{}}
}

func (b *Builder) Build() (*WorkerConfig, error) {
	if b.c.workspace.IsNil() {
		return nil, errors.New("workspace is required")
	}

	if b.c.createdAt.IsZero() {
		b.c.createdAt = time.Now()
	}
	if b.c.updatedAt.IsZero() {
		b.c.updatedAt = b.c.createdAt
	}

	return b.c, nil
}

func (b *Builder) MustBuild() *WorkerConfig {
	r, err := b.Build()
	if err != nil {
		panic(err)
	}
	return r
}

func (b *Builder) Workspace(workspace id.WorkspaceID) *Builder {
	b.c.workspace = workspace
	return b
}

func (b *Builder) MachineType(machineType *string) *Builder {
	b.c.machineType = cloneString(machineType)
	return b
}

func (b *Builder) ComputeCpuMilli(computeCpuMilli *int) *Builder {
	b.c.computeCpuMilli = cloneInt(computeCpuMilli)
	return b
}

func (b *Builder) ComputeMemoryMib(computeMemoryMib *int) *Builder {
	b.c.computeMemoryMib = cloneInt(computeMemoryMib)
	return b
}

func (b *Builder) BootDiskSizeGB(bootDiskSizeGB *int) *Builder {
	b.c.bootDiskSizeGB = cloneInt(bootDiskSizeGB)
	return b
}

func (b *Builder) TaskCount(taskCount *int) *Builder {
	b.c.taskCount = cloneInt(taskCount)
	return b
}

func (b *Builder) MaxConcurrency(maxConcurrency *int) *Builder {
	b.c.maxConcurrency = cloneInt(maxConcurrency)
	return b
}

func (b *Builder) ThreadPoolSize(threadPoolSize *int) *Builder {
	b.c.threadPoolSize = cloneInt(threadPoolSize)
	return b
}

func (b *Builder) ChannelBufferSize(channelBufferSize *int) *Builder {
	b.c.channelBufferSize = cloneInt(channelBufferSize)
	return b
}

func (b *Builder) FeatureFlushThreshold(featureFlushThreshold *int) *Builder {
	b.c.featureFlushThreshold = cloneInt(featureFlushThreshold)
	return b
}

func (b *Builder) NodeStatusPropagationDelayMilli(nodeStatusDelayMilli *int) *Builder {
	b.c.nodeStatusDelayMilli = cloneInt(nodeStatusDelayMilli)
	return b
}

func (b *Builder) CreatedAt(createdAt time.Time) *Builder {
	b.c.createdAt = createdAt
	return b
}

func (b *Builder) UpdatedAt(updatedAt time.Time) *Builder {
	b.c.updatedAt = updatedAt
	return b
}
