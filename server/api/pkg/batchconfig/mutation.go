package batchconfig

type Mutations struct {
	machineType             *string
	computeCpuMilli         *int
	computeMemoryMib        *int
	bootDiskSizeGB          *int
	taskCount               *int
	maxConcurrency          *int
	threadPoolSize          *int
	channelBufferSize       *int
	featureFlushThreshold   *int
	nodeStatusDelayMilli    *int
}

func (m *Mutations) MachineType() *string { return m.machineType }
func (m *Mutations) ComputeCpuMilli() *int { return m.computeCpuMilli }
func (m *Mutations) ComputeMemoryMib() *int { return m.computeMemoryMib }
func (m *Mutations) BootDiskSizeGB() *int { return m.bootDiskSizeGB }
func (m *Mutations) TaskCount() *int { return m.taskCount }
func (m *Mutations) MaxConcurrency() *int { return m.maxConcurrency }
func (m *Mutations) ThreadPoolSize() *int { return m.threadPoolSize }
func (m *Mutations) ChannelBufferSize() *int { return m.channelBufferSize }
func (m *Mutations) FeatureFlushThreshold() *int { return m.featureFlushThreshold }
func (m *Mutations) NodeStatusDelayMilli() *int { return m.nodeStatusDelayMilli }

func FromInput(machineType, computeCpuMilli, computeMemoryMib, bootDiskSizeGB, taskCount, maxConcurrency, threadPoolSize, channelBufferSize, featureFlushThreshold, nodeStatusDelayMilli *int) *Mutations {
	return &Mutations{
		computeCpuMilli:         computeCpuMilli,
		computeMemoryMib:        computeMemoryMib,
		bootDiskSizeGB:          bootDiskSizeGB,
		taskCount:               taskCount,
		maxConcurrency:          maxConcurrency,
		threadPoolSize:          threadPoolSize,
		channelBufferSize:       channelBufferSize,
		featureFlushThreshold:   featureFlushThreshold,
		nodeStatusDelayMilli:    nodeStatusDelayMilli,
	}
}
