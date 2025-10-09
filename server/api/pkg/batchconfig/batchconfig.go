package batchconfig

import (
	"time"
)

// ConfigTier represents the access level required to modify a configuration parameter
type ConfigTier string

const (
	ConfigTierA ConfigTier = "A" // High priority, low risk - VIP/User accessible
	ConfigTierB ConfigTier = "B" // Medium priority, requires guardrails - Admin/VIP
	ConfigTierC ConfigTier = "C" // Admin-only, high risk
)

// BatchConfig represents workspace-specific batch worker configuration overrides
type BatchConfig struct {
	id          ID
	workspaceID WorkspaceID
	createdAt   time.Time
	updatedAt   time.Time
	createdBy   string
	updatedBy   string

	// Tier A parameters (high priority, low risk)
	computeCpuMilli       *int // vCPU milli units per task
	computeMemoryMib      *int // Memory MiB per task
	bootDiskSizeGB        *int // Boot disk size
	maxConcurrency        *int // Concurrency within worker
	threadPoolSize        *int // Thread pool size
	channelBufferSize     *int // Channel buffer for runtime
	featureFlushThreshold *int // Data flush threshold

	// Tier B parameters (medium priority, requires guardrails)
	machineType                  *string // GCE machine type
	taskCount                    *int    // Tasks per job
	nodeStatusPropagationDelayMS *int    // Status propagation delay

	// Tier C parameters (admin-only, high risk)
	bootDiskType     *string  // Boot disk type (pd-balanced, etc.)
	imageURL         *string  // Container image URL
	binaryPath       *string  // Entry point binary
	allowedLocations []string // Location policy for Batch

	// Audit fields
	changeHistory []ConfigChange
}

// ConfigChange records a configuration change for audit purposes
type ConfigChange struct {
	Timestamp time.Time
	ChangedBy string
	FieldName string
	OldValue  interface{}
	NewValue  interface{}
}

// Getters
func (b *BatchConfig) ID() ID {
	return b.id
}

func (b *BatchConfig) WorkspaceID() WorkspaceID {
	return b.workspaceID
}

func (b *BatchConfig) CreatedAt() time.Time {
	return b.createdAt
}

func (b *BatchConfig) UpdatedAt() time.Time {
	return b.updatedAt
}

func (b *BatchConfig) CreatedBy() string {
	return b.createdBy
}

func (b *BatchConfig) UpdatedBy() string {
	return b.updatedBy
}

// Tier A Getters
func (b *BatchConfig) ComputeCpuMilli() *int {
	return b.computeCpuMilli
}

func (b *BatchConfig) ComputeMemoryMib() *int {
	return b.computeMemoryMib
}

func (b *BatchConfig) BootDiskSizeGB() *int {
	return b.bootDiskSizeGB
}

func (b *BatchConfig) MaxConcurrency() *int {
	return b.maxConcurrency
}

func (b *BatchConfig) ThreadPoolSize() *int {
	return b.threadPoolSize
}

func (b *BatchConfig) ChannelBufferSize() *int {
	return b.channelBufferSize
}

func (b *BatchConfig) FeatureFlushThreshold() *int {
	return b.featureFlushThreshold
}

// Tier B Getters
func (b *BatchConfig) MachineType() *string {
	return b.machineType
}

func (b *BatchConfig) TaskCount() *int {
	return b.taskCount
}

func (b *BatchConfig) NodeStatusPropagationDelayMS() *int {
	return b.nodeStatusPropagationDelayMS
}

// Tier C Getters
func (b *BatchConfig) BootDiskType() *string {
	return b.bootDiskType
}

func (b *BatchConfig) ImageURL() *string {
	return b.imageURL
}

func (b *BatchConfig) BinaryPath() *string {
	return b.binaryPath
}

func (b *BatchConfig) AllowedLocations() []string {
	return b.allowedLocations
}

func (b *BatchConfig) ChangeHistory() []ConfigChange {
	return b.changeHistory
}

// Setters with audit logging

// SetComputeCpuMilli sets CPU allocation with audit trail
func (b *BatchConfig) SetComputeCpuMilli(value *int, changedBy string) {
	oldValue := b.computeCpuMilli
	b.computeCpuMilli = value
	b.recordChange("ComputeCpuMilli", oldValue, value, changedBy)
}

// SetComputeMemoryMib sets memory allocation with audit trail
func (b *BatchConfig) SetComputeMemoryMib(value *int, changedBy string) {
	oldValue := b.computeMemoryMib
	b.computeMemoryMib = value
	b.recordChange("ComputeMemoryMib", oldValue, value, changedBy)
}

// SetBootDiskSizeGB sets boot disk size with audit trail
func (b *BatchConfig) SetBootDiskSizeGB(value *int, changedBy string) {
	oldValue := b.bootDiskSizeGB
	b.bootDiskSizeGB = value
	b.recordChange("BootDiskSizeGB", oldValue, value, changedBy)
}

// SetMaxConcurrency sets max concurrency with audit trail
func (b *BatchConfig) SetMaxConcurrency(value *int, changedBy string) {
	oldValue := b.maxConcurrency
	b.maxConcurrency = value
	b.recordChange("MaxConcurrency", oldValue, value, changedBy)
}

// SetThreadPoolSize sets thread pool size with audit trail
func (b *BatchConfig) SetThreadPoolSize(value *int, changedBy string) {
	oldValue := b.threadPoolSize
	b.threadPoolSize = value
	b.recordChange("ThreadPoolSize", oldValue, value, changedBy)
}

// SetChannelBufferSize sets channel buffer size with audit trail
func (b *BatchConfig) SetChannelBufferSize(value *int, changedBy string) {
	oldValue := b.channelBufferSize
	b.channelBufferSize = value
	b.recordChange("ChannelBufferSize", oldValue, value, changedBy)
}

// SetFeatureFlushThreshold sets feature flush threshold with audit trail
func (b *BatchConfig) SetFeatureFlushThreshold(value *int, changedBy string) {
	oldValue := b.featureFlushThreshold
	b.featureFlushThreshold = value
	b.recordChange("FeatureFlushThreshold", oldValue, value, changedBy)
}

// SetMachineType sets machine type with audit trail
func (b *BatchConfig) SetMachineType(value *string, changedBy string) {
	oldValue := b.machineType
	b.machineType = value
	b.recordChange("MachineType", oldValue, value, changedBy)
}

// SetTaskCount sets task count with audit trail
func (b *BatchConfig) SetTaskCount(value *int, changedBy string) {
	oldValue := b.taskCount
	b.taskCount = value
	b.recordChange("TaskCount", oldValue, value, changedBy)
}

// SetNodeStatusPropagationDelayMS sets node status propagation delay with audit trail
func (b *BatchConfig) SetNodeStatusPropagationDelayMS(value *int, changedBy string) {
	oldValue := b.nodeStatusPropagationDelayMS
	b.nodeStatusPropagationDelayMS = value
	b.recordChange("NodeStatusPropagationDelayMS", oldValue, value, changedBy)
}

// SetBootDiskType sets boot disk type with audit trail (Admin only)
func (b *BatchConfig) SetBootDiskType(value *string, changedBy string) {
	oldValue := b.bootDiskType
	b.bootDiskType = value
	b.recordChange("BootDiskType", oldValue, value, changedBy)
}

// SetImageURL sets container image URL with audit trail (Admin only)
func (b *BatchConfig) SetImageURL(value *string, changedBy string) {
	oldValue := b.imageURL
	b.imageURL = value
	b.recordChange("ImageURL", oldValue, value, changedBy)
}

// SetBinaryPath sets binary path with audit trail (Admin only)
func (b *BatchConfig) SetBinaryPath(value *string, changedBy string) {
	oldValue := b.binaryPath
	b.binaryPath = value
	b.recordChange("BinaryPath", oldValue, value, changedBy)
}

// SetAllowedLocations sets allowed locations with audit trail (Admin only)
func (b *BatchConfig) SetAllowedLocations(value []string, changedBy string) {
	oldValue := b.allowedLocations
	b.allowedLocations = value
	b.recordChange("AllowedLocations", oldValue, value, changedBy)
}

// recordChange records a configuration change in the audit log
func (b *BatchConfig) recordChange(fieldName string, oldValue, newValue interface{}, changedBy string) {
	b.updatedAt = time.Now()
	b.updatedBy = changedBy

	change := ConfigChange{
		Timestamp: time.Now(),
		ChangedBy: changedBy,
		FieldName: fieldName,
		OldValue:  oldValue,
		NewValue:  newValue,
	}

	b.changeHistory = append(b.changeHistory, change)
}
