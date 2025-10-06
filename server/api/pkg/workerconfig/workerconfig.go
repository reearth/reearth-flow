package workerconfig

import (
	"errors"
	"time"
)

// Validation constants
const (
	MinCPUMilli              = 100    // 0.1 CPU
	MaxCPUMilli              = 96000  // 96 CPUs
	MinMemoryMib             = 128    // 128 MB
	MaxMemoryMib             = 624000 // 624 GB
	MinBootDiskSizeGB        = 10     // 10 GB
	MaxBootDiskSizeGB        = 10000  // 10 TB
	MinChannelBuffer         = 1
	MaxChannelBuffer         = 10000
	MinMaxConcurrency        = 1
	MaxMaxConcurrency        = 100
	MinFeatureFlushThreshold = 1
	MaxFeatureFlushThreshold = 100000
)

// Valid disk types for GCP
var ValidDiskTypes = []string{
	"pd-standard",
	"pd-balanced",
	"pd-ssd",
}

// Errors
var (
	ErrInvalidCPU            = errors.New("CPU must be between 100 and 96000 millicores")
	ErrInvalidMemory         = errors.New("memory must be between 128 and 624000 MiB")
	ErrInvalidDiskSize       = errors.New("disk size must be between 10 and 10000 GB")
	ErrInvalidDiskType       = errors.New("invalid disk type, must be one of: pd-standard, pd-balanced, pd-ssd")
	ErrInvalidChannelBuffer  = errors.New("channel buffer size must be between 1 and 10000")
	ErrInvalidMaxConcurrency = errors.New("max concurrency must be between 1 and 100")
	ErrInvalidFeatureFlush   = errors.New("feature flush threshold must be between 1 and 100000")
	ErrWorkspaceIDRequired   = errors.New("workspace ID is required")
)

// WorkerConfig represents worker compute resource configuration for a workspace
type WorkerConfig struct {
	id                    ID
	workspaceID           WorkspaceID
	bootDiskSizeGB        *int
	bootDiskType          *string
	channelBufferSize     *int
	computeCpuMilli       *int
	computeMemoryMib      *int
	featureFlushThreshold *int
	imageURL              *string
	machineType           *string
	maxConcurrency        *int
	createdAt             time.Time
	updatedAt             time.Time
}

// Getters

func (w *WorkerConfig) ID() ID {
	return w.id
}

func (w *WorkerConfig) WorkspaceID() WorkspaceID {
	return w.workspaceID
}

func (w *WorkerConfig) BootDiskSizeGB() *int {
	return w.bootDiskSizeGB
}

func (w *WorkerConfig) BootDiskType() *string {
	return w.bootDiskType
}

func (w *WorkerConfig) ChannelBufferSize() *int {
	return w.channelBufferSize
}

func (w *WorkerConfig) ComputeCpuMilli() *int {
	return w.computeCpuMilli
}

func (w *WorkerConfig) ComputeMemoryMib() *int {
	return w.computeMemoryMib
}

func (w *WorkerConfig) FeatureFlushThreshold() *int {
	return w.featureFlushThreshold
}

func (w *WorkerConfig) ImageURL() *string {
	return w.imageURL
}

func (w *WorkerConfig) MachineType() *string {
	return w.machineType
}

func (w *WorkerConfig) MaxConcurrency() *int {
	return w.maxConcurrency
}

func (w *WorkerConfig) CreatedAt() time.Time {
	return w.createdAt
}

func (w *WorkerConfig) UpdatedAt() time.Time {
	return w.updatedAt
}

// Setters with validation

func (w *WorkerConfig) SetBootDiskSizeGB(size *int) error {
	if size != nil && (*size < MinBootDiskSizeGB || *size > MaxBootDiskSizeGB) {
		return ErrInvalidDiskSize
	}
	w.bootDiskSizeGB = size
	w.updatedAt = time.Now()
	return nil
}

func (w *WorkerConfig) SetBootDiskType(diskType *string) error {
	if diskType != nil {
		valid := false
		for _, t := range ValidDiskTypes {
			if *diskType == t {
				valid = true
				break
			}
		}
		if !valid {
			return ErrInvalidDiskType
		}
	}
	w.bootDiskType = diskType
	w.updatedAt = time.Now()
	return nil
}

func (w *WorkerConfig) SetChannelBufferSize(size *int) error {
	if size != nil && (*size < MinChannelBuffer || *size > MaxChannelBuffer) {
		return ErrInvalidChannelBuffer
	}
	w.channelBufferSize = size
	w.updatedAt = time.Now()
	return nil
}

func (w *WorkerConfig) SetComputeCpuMilli(cpu *int) error {
	if cpu != nil && (*cpu < MinCPUMilli || *cpu > MaxCPUMilli) {
		return ErrInvalidCPU
	}
	w.computeCpuMilli = cpu
	w.updatedAt = time.Now()
	return nil
}

func (w *WorkerConfig) SetComputeMemoryMib(memory *int) error {
	if memory != nil && (*memory < MinMemoryMib || *memory > MaxMemoryMib) {
		return ErrInvalidMemory
	}
	w.computeMemoryMib = memory
	w.updatedAt = time.Now()
	return nil
}

func (w *WorkerConfig) SetFeatureFlushThreshold(threshold *int) error {
	if threshold != nil && (*threshold < MinFeatureFlushThreshold || *threshold > MaxFeatureFlushThreshold) {
		return ErrInvalidFeatureFlush
	}
	w.featureFlushThreshold = threshold
	w.updatedAt = time.Now()
	return nil
}

func (w *WorkerConfig) SetImageURL(url *string) {
	w.imageURL = url
	w.updatedAt = time.Now()
}

func (w *WorkerConfig) SetMachineType(machineType *string) {
	w.machineType = machineType
	w.updatedAt = time.Now()
}

func (w *WorkerConfig) SetMaxConcurrency(concurrency *int) error {
	if concurrency != nil && (*concurrency < MinMaxConcurrency || *concurrency > MaxMaxConcurrency) {
		return ErrInvalidMaxConcurrency
	}
	w.maxConcurrency = concurrency
	w.updatedAt = time.Now()
	return nil
}

// Helper methods

// HasOverrides returns true if any configuration value is set (not using defaults)
func (w *WorkerConfig) HasOverrides() bool {
	return w.bootDiskSizeGB != nil ||
		w.bootDiskType != nil ||
		w.channelBufferSize != nil ||
		w.computeCpuMilli != nil ||
		w.computeMemoryMib != nil ||
		w.featureFlushThreshold != nil ||
		w.imageURL != nil ||
		w.machineType != nil ||
		w.maxConcurrency != nil
}

// ResetField resets a specific field to use global defaults
func (w *WorkerConfig) ResetField(fieldName string) error {
	switch fieldName {
	case "bootDiskSizeGB":
		w.bootDiskSizeGB = nil
	case "bootDiskType":
		w.bootDiskType = nil
	case "channelBufferSize":
		w.channelBufferSize = nil
	case "computeCpuMilli":
		w.computeCpuMilli = nil
	case "computeMemoryMib":
		w.computeMemoryMib = nil
	case "featureFlushThreshold":
		w.featureFlushThreshold = nil
	case "imageURL":
		w.imageURL = nil
	case "machineType":
		w.machineType = nil
	case "maxConcurrency":
		w.maxConcurrency = nil
	default:
		return errors.New("unknown field: " + fieldName)
	}
	w.updatedAt = time.Now()
	return nil
}

// ResetAll resets all fields to use global defaults
func (w *WorkerConfig) ResetAll() {
	w.bootDiskSizeGB = nil
	w.bootDiskType = nil
	w.channelBufferSize = nil
	w.computeCpuMilli = nil
	w.computeMemoryMib = nil
	w.featureFlushThreshold = nil
	w.imageURL = nil
	w.machineType = nil
	w.maxConcurrency = nil
	w.updatedAt = time.Now()
}
