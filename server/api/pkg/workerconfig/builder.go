package workerconfig

import "time"

type Builder struct {
	w *WorkerConfig
}

func New() *Builder {
	return &Builder{w: &WorkerConfig{}}
}

func (b *Builder) Build() (*WorkerConfig, error) {
	if b.w.id.IsNil() {
		return nil, ErrInvalidID
	}
	if b.w.workspaceID.IsNil() {
		return nil, ErrWorkspaceIDRequired
	}
	if b.w.createdAt.IsZero() {
		b.w.createdAt = time.Now()
	}
	if b.w.updatedAt.IsZero() {
		b.w.updatedAt = b.w.createdAt
	}
	return b.w, nil
}

func (b *Builder) MustBuild() *WorkerConfig {
	w, err := b.Build()
	if err != nil {
		panic(err)
	}
	return w
}

func (b *Builder) ID(id ID) *Builder {
	b.w.id = id
	return b
}

func (b *Builder) NewID() *Builder {
	b.w.id = NewID()
	return b
}

func (b *Builder) WorkspaceID(wsID WorkspaceID) *Builder {
	b.w.workspaceID = wsID
	return b
}

func (b *Builder) BootDiskSizeGB(size *int) *Builder {
	b.w.bootDiskSizeGB = size
	return b
}

func (b *Builder) BootDiskType(diskType *string) *Builder {
	b.w.bootDiskType = diskType
	return b
}

func (b *Builder) ChannelBufferSize(size *int) *Builder {
	b.w.channelBufferSize = size
	return b
}

func (b *Builder) ComputeCpuMilli(cpu *int) *Builder {
	b.w.computeCpuMilli = cpu
	return b
}

func (b *Builder) ComputeMemoryMib(memory *int) *Builder {
	b.w.computeMemoryMib = memory
	return b
}

func (b *Builder) FeatureFlushThreshold(threshold *int) *Builder {
	b.w.featureFlushThreshold = threshold
	return b
}

func (b *Builder) ImageURL(url *string) *Builder {
	b.w.imageURL = url
	return b
}

func (b *Builder) MachineType(machineType *string) *Builder {
	b.w.machineType = machineType
	return b
}

func (b *Builder) MaxConcurrency(concurrency *int) *Builder {
	b.w.maxConcurrency = concurrency
	return b
}

func (b *Builder) CreatedAt(t time.Time) *Builder {
	b.w.createdAt = t
	return b
}

func (b *Builder) UpdatedAt(t time.Time) *Builder {
	b.w.updatedAt = t
	return b
}
