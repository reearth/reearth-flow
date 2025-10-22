package batchconfig

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
)

type WorkerConfig struct {
	workspace             id.WorkspaceID
	machineType           *string
	computeCpuMilli       *int
	computeMemoryMib      *int
	bootDiskSizeGB        *int
	taskCount             *int
	maxConcurrency        *int
	threadPoolSize        *int
	channelBufferSize     *int
	featureFlushThreshold *int
	nodeStatusDelayMilli  *int
	createdAt             time.Time
	updatedAt             time.Time
}

func New(workspace id.WorkspaceID) *WorkerConfig {
	now := time.Now()
	return &WorkerConfig{
		workspace: workspace,
		createdAt: now,
		updatedAt: now,
	}
}

func (c *WorkerConfig) Workspace() id.WorkspaceID {
	return c.workspace
}

func (c *WorkerConfig) MachineType() *string {
	return c.machineType
}

func (c *WorkerConfig) ComputeCpuMilli() *int {
	return c.computeCpuMilli
}

func (c *WorkerConfig) ComputeMemoryMib() *int {
	return c.computeMemoryMib
}

func (c *WorkerConfig) BootDiskSizeGB() *int {
	return c.bootDiskSizeGB
}

func (c *WorkerConfig) TaskCount() *int {
	return c.taskCount
}

func (c *WorkerConfig) MaxConcurrency() *int {
	return c.maxConcurrency
}

func (c *WorkerConfig) ThreadPoolSize() *int {
	return c.threadPoolSize
}

func (c *WorkerConfig) ChannelBufferSize() *int {
	return c.channelBufferSize
}

func (c *WorkerConfig) FeatureFlushThreshold() *int {
	return c.featureFlushThreshold
}

func (c *WorkerConfig) NodeStatusPropagationDelayMilli() *int {
	return c.nodeStatusDelayMilli
}

func (c *WorkerConfig) CreatedAt() time.Time {
	return c.createdAt
}

func (c *WorkerConfig) UpdatedAt() time.Time {
	return c.updatedAt
}

func (c *WorkerConfig) touch() {
	c.updatedAt = time.Now()
}

func (c *WorkerConfig) SetMachineType(value *string) {
	c.machineType = cloneString(value)
	c.touch()
}

func (c *WorkerConfig) SetComputeCpuMilli(value *int) {
	c.computeCpuMilli = cloneInt(value)
	c.touch()
}

func (c *WorkerConfig) SetComputeMemoryMib(value *int) {
	c.computeMemoryMib = cloneInt(value)
	c.touch()
}

func (c *WorkerConfig) SetBootDiskSizeGB(value *int) {
	c.bootDiskSizeGB = cloneInt(value)
	c.touch()
}

func (c *WorkerConfig) SetTaskCount(value *int) {
	c.taskCount = cloneInt(value)
	c.touch()
}

func (c *WorkerConfig) SetMaxConcurrency(value *int) {
	c.maxConcurrency = cloneInt(value)
	c.touch()
}

func (c *WorkerConfig) SetThreadPoolSize(value *int) {
	c.threadPoolSize = cloneInt(value)
	c.touch()
}

func (c *WorkerConfig) SetChannelBufferSize(value *int) {
	c.channelBufferSize = cloneInt(value)
	c.touch()
}

func (c *WorkerConfig) SetFeatureFlushThreshold(value *int) {
	c.featureFlushThreshold = cloneInt(value)
	c.touch()
}

func (c *WorkerConfig) SetNodeStatusPropagationDelayMilli(value *int) {
	c.nodeStatusDelayMilli = cloneInt(value)
	c.touch()
}

func (c *WorkerConfig) ReplaceTimestamps(createdAt, updatedAt time.Time) {
	if !createdAt.IsZero() {
		c.createdAt = createdAt
	}
	if !updatedAt.IsZero() {
		c.updatedAt = updatedAt
	}
}

func (c *WorkerConfig) IsEmpty() bool {
	return c.machineType == nil &&
		c.computeCpuMilli == nil &&
		c.computeMemoryMib == nil &&
		c.bootDiskSizeGB == nil &&
		c.taskCount == nil &&
		c.maxConcurrency == nil &&
		c.threadPoolSize == nil &&
		c.channelBufferSize == nil &&
		c.featureFlushThreshold == nil &&
		c.nodeStatusDelayMilli == nil
}

func Clone(c *WorkerConfig) *WorkerConfig {
	if c == nil {
		return nil
	}
	return &WorkerConfig{
		workspace:             c.workspace,
		machineType:           cloneString(c.machineType),
		computeCpuMilli:       cloneInt(c.computeCpuMilli),
		computeMemoryMib:      cloneInt(c.computeMemoryMib),
		bootDiskSizeGB:        cloneInt(c.bootDiskSizeGB),
		taskCount:             cloneInt(c.taskCount),
		maxConcurrency:        cloneInt(c.maxConcurrency),
		threadPoolSize:        cloneInt(c.threadPoolSize),
		channelBufferSize:     cloneInt(c.channelBufferSize),
		featureFlushThreshold: cloneInt(c.featureFlushThreshold),
		nodeStatusDelayMilli:  cloneInt(c.nodeStatusDelayMilli),
		createdAt:             c.createdAt,
		updatedAt:             c.updatedAt,
	}
}

func cloneInt(v *int) *int {
	if v == nil {
		return nil
	}
	clone := *v
	return &clone
}

func cloneString(v *string) *string {
	if v == nil {
		return nil
	}
	clone := *v
	return &clone
}
