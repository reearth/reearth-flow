package batchconfig

import (
	"errors"
	"fmt"
	"regexp"
	"strings"
)

var (
	ErrValidationFailed     = errors.New("validation failed")
	ErrValueOutOfRange      = errors.New("value out of allowed range")
	ErrInvalidMachineType   = errors.New("invalid machine type")
	ErrInvalidBootDiskType  = errors.New("invalid boot disk type")
	ErrInvalidLocation      = errors.New("invalid location")
	ErrMemoryExceedsMachine = errors.New("memory exceeds machine type capacity")
	ErrCPUExceedsMachine    = errors.New("CPU exceeds machine type capacity")
)

// Validator provides validation logic for batch configuration parameters
type Validator struct{}

// NewValidator creates a new configuration validator
func NewValidator() *Validator {
	return &Validator{}
}

// Validation constraints based on GCP Batch limits and worker runtime tolerance
const (
	// Tier A constraints
	MinComputeCpuMilli       = 250    // Minimum 0.25 vCPU
	MaxComputeCpuMilli       = 96000  // Maximum 96 vCPU (typical max for most machine types)
	MinComputeMemoryMib      = 256    // Minimum 256 MiB
	MaxComputeMemoryMib      = 786432 // Maximum 768 GiB
	MinBootDiskSizeGB        = 10     // Minimum boot disk size
	MaxBootDiskSizeGB        = 10000  // Maximum boot disk size (10 TB)
	MinMaxConcurrency        = 1      // Minimum concurrency
	MaxMaxConcurrency        = 128    // Maximum concurrency
	MinThreadPoolSize        = 1      // Minimum thread pool size
	MaxThreadPoolSize        = 256    // Maximum thread pool size
	MinChannelBufferSize     = 1      // Minimum channel buffer
	MaxChannelBufferSize     = 10000  // Maximum channel buffer
	MinFeatureFlushThreshold = 1      // Minimum flush threshold
	MaxFeatureFlushThreshold = 100000 // Maximum flush threshold

	// Tier B constraints
	MinTaskCount                    = 1     // Minimum tasks per job
	MaxTaskCount                    = 10000 // Maximum tasks per job
	MinNodeStatusPropagationDelayMS = 100   // Minimum delay (100ms)
	MaxNodeStatusPropagationDelayMS = 60000 // Maximum delay (60s)
)

// Allowed values for Tier B/C parameters
var (
	AllowedMachineTypes = []string{
		// E2 series (cost-effective)
		"e2-micro", "e2-small", "e2-medium",
		"e2-standard-2", "e2-standard-4", "e2-standard-8", "e2-standard-16", "e2-standard-32",
		"e2-highmem-2", "e2-highmem-4", "e2-highmem-8", "e2-highmem-16",
		"e2-highcpu-2", "e2-highcpu-4", "e2-highcpu-8", "e2-highcpu-16", "e2-highcpu-32",
		// N1 series
		"n1-standard-1", "n1-standard-2", "n1-standard-4", "n1-standard-8", "n1-standard-16", "n1-standard-32", "n1-standard-64", "n1-standard-96",
		"n1-highmem-2", "n1-highmem-4", "n1-highmem-8", "n1-highmem-16", "n1-highmem-32", "n1-highmem-64", "n1-highmem-96",
		"n1-highcpu-2", "n1-highcpu-4", "n1-highcpu-8", "n1-highcpu-16", "n1-highcpu-32", "n1-highcpu-64", "n1-highcpu-96",
		// N2 series
		"n2-standard-2", "n2-standard-4", "n2-standard-8", "n2-standard-16", "n2-standard-32", "n2-standard-48", "n2-standard-64", "n2-standard-80", "n2-standard-96", "n2-standard-128",
		"n2-highmem-2", "n2-highmem-4", "n2-highmem-8", "n2-highmem-16", "n2-highmem-32", "n2-highmem-48", "n2-highmem-64", "n2-highmem-80", "n2-highmem-96", "n2-highmem-128",
		"n2-highcpu-2", "n2-highcpu-4", "n2-highcpu-8", "n2-highcpu-16", "n2-highcpu-32", "n2-highcpu-48", "n2-highcpu-64", "n2-highcpu-80", "n2-highcpu-96",
		// N2D series
		"n2d-standard-2", "n2d-standard-4", "n2d-standard-8", "n2d-standard-16", "n2d-standard-32", "n2d-standard-48", "n2d-standard-64", "n2d-standard-80", "n2d-standard-96", "n2d-standard-128", "n2d-standard-224",
		"n2d-highmem-2", "n2d-highmem-4", "n2d-highmem-8", "n2d-highmem-16", "n2d-highmem-32", "n2d-highmem-48", "n2d-highmem-64", "n2d-highmem-80", "n2d-highmem-96",
		"n2d-highcpu-2", "n2d-highcpu-4", "n2d-highcpu-8", "n2d-highcpu-16", "n2d-highcpu-32", "n2d-highcpu-48", "n2d-highcpu-64", "n2d-highcpu-80", "n2d-highcpu-96", "n2d-highcpu-128", "n2d-highcpu-224",
		// C2 series (compute-optimized)
		"c2-standard-4", "c2-standard-8", "c2-standard-16", "c2-standard-30", "c2-standard-60",
		// C2D series
		"c2d-standard-2", "c2d-standard-4", "c2d-standard-8", "c2d-standard-16", "c2d-standard-32", "c2d-standard-56", "c2d-standard-112",
		"c2d-highmem-2", "c2d-highmem-4", "c2d-highmem-8", "c2d-highmem-16", "c2d-highmem-32", "c2d-highmem-56", "c2d-highmem-112",
		"c2d-highcpu-2", "c2d-highcpu-4", "c2d-highcpu-8", "c2d-highcpu-16", "c2d-highcpu-32", "c2d-highcpu-56", "c2d-highcpu-112",
	}

	AllowedBootDiskTypes = []string{
		"pd-standard", // Standard persistent disk
		"pd-balanced", // Balanced persistent disk (default)
		"pd-ssd",      // SSD persistent disk
		"pd-extreme",  // Extreme persistent disk
	}

	// GCP region pattern
	gcpRegionPattern = regexp.MustCompile(`^[a-z]+-[a-z]+\d+(-[a-z])?$`)
)

// ValidateTierAConfig validates all Tier A configuration parameters
func (v *Validator) ValidateTierAConfig(config *BatchConfig) error {
	if config.ComputeCpuMilli() != nil {
		if err := v.ValidateComputeCpuMilli(*config.ComputeCpuMilli()); err != nil {
			return err
		}
	}
	if config.ComputeMemoryMib() != nil {
		if err := v.ValidateComputeMemoryMib(*config.ComputeMemoryMib()); err != nil {
			return err
		}
	}
	if config.BootDiskSizeGB() != nil {
		if err := v.ValidateBootDiskSizeGB(*config.BootDiskSizeGB()); err != nil {
			return err
		}
	}
	if config.MaxConcurrency() != nil {
		if err := v.ValidateMaxConcurrency(*config.MaxConcurrency()); err != nil {
			return err
		}
	}
	if config.ThreadPoolSize() != nil {
		if err := v.ValidateThreadPoolSize(*config.ThreadPoolSize()); err != nil {
			return err
		}
	}
	if config.ChannelBufferSize() != nil {
		if err := v.ValidateChannelBufferSize(*config.ChannelBufferSize()); err != nil {
			return err
		}
	}
	if config.FeatureFlushThreshold() != nil {
		if err := v.ValidateFeatureFlushThreshold(*config.FeatureFlushThreshold()); err != nil {
			return err
		}
	}
	return nil
}

// ValidateTierBConfig validates all Tier B configuration parameters
func (v *Validator) ValidateTierBConfig(config *BatchConfig) error {
	if config.MachineType() != nil {
		if err := v.ValidateMachineType(*config.MachineType()); err != nil {
			return err
		}
	}
	if config.TaskCount() != nil {
		if err := v.ValidateTaskCount(*config.TaskCount()); err != nil {
			return err
		}
	}
	if config.NodeStatusPropagationDelayMS() != nil {
		if err := v.ValidateNodeStatusPropagationDelayMS(*config.NodeStatusPropagationDelayMS()); err != nil {
			return err
		}
	}
	return nil
}

// ValidateTierCConfig validates all Tier C configuration parameters
func (v *Validator) ValidateTierCConfig(config *BatchConfig) error {
	if config.BootDiskType() != nil {
		if err := v.ValidateBootDiskType(*config.BootDiskType()); err != nil {
			return err
		}
	}
	if config.ImageURL() != nil {
		if err := v.ValidateImageURL(*config.ImageURL()); err != nil {
			return err
		}
	}
	if config.BinaryPath() != nil {
		if err := v.ValidateBinaryPath(*config.BinaryPath()); err != nil {
			return err
		}
	}
	if config.AllowedLocations() != nil {
		if err := v.ValidateAllowedLocations(config.AllowedLocations()); err != nil {
			return err
		}
	}
	return nil
}

// ValidateFullConfig validates the entire configuration including cross-field validation
func (v *Validator) ValidateFullConfig(config *BatchConfig) error {
	if err := v.ValidateTierAConfig(config); err != nil {
		return err
	}
	if err := v.ValidateTierBConfig(config); err != nil {
		return err
	}
	if err := v.ValidateTierCConfig(config); err != nil {
		return err
	}

	// Cross-field validation
	if config.MachineType() != nil && config.ComputeMemoryMib() != nil {
		if err := v.ValidateMachineTypeMemoryCompatibility(*config.MachineType(), *config.ComputeMemoryMib()); err != nil {
			return err
		}
	}
	if config.MachineType() != nil && config.ComputeCpuMilli() != nil {
		if err := v.ValidateMachineTypeCPUCompatibility(*config.MachineType(), *config.ComputeCpuMilli()); err != nil {
			return err
		}
	}

	return nil
}

// Individual field validators

func (v *Validator) ValidateComputeCpuMilli(value int) error {
	if value < MinComputeCpuMilli || value > MaxComputeCpuMilli {
		return fmt.Errorf("%w: ComputeCpuMilli must be between %d and %d, got %d", ErrValueOutOfRange, MinComputeCpuMilli, MaxComputeCpuMilli, value)
	}
	return nil
}

func (v *Validator) ValidateComputeMemoryMib(value int) error {
	if value < MinComputeMemoryMib || value > MaxComputeMemoryMib {
		return fmt.Errorf("%w: ComputeMemoryMib must be between %d and %d, got %d", ErrValueOutOfRange, MinComputeMemoryMib, MaxComputeMemoryMib, value)
	}
	return nil
}

func (v *Validator) ValidateBootDiskSizeGB(value int) error {
	if value < MinBootDiskSizeGB || value > MaxBootDiskSizeGB {
		return fmt.Errorf("%w: BootDiskSizeGB must be between %d and %d, got %d", ErrValueOutOfRange, MinBootDiskSizeGB, MaxBootDiskSizeGB, value)
	}
	return nil
}

func (v *Validator) ValidateMaxConcurrency(value int) error {
	if value < MinMaxConcurrency || value > MaxMaxConcurrency {
		return fmt.Errorf("%w: MaxConcurrency must be between %d and %d, got %d", ErrValueOutOfRange, MinMaxConcurrency, MaxMaxConcurrency, value)
	}
	return nil
}

func (v *Validator) ValidateThreadPoolSize(value int) error {
	if value < MinThreadPoolSize || value > MaxThreadPoolSize {
		return fmt.Errorf("%w: ThreadPoolSize must be between %d and %d, got %d", ErrValueOutOfRange, MinThreadPoolSize, MaxThreadPoolSize, value)
	}
	return nil
}

func (v *Validator) ValidateChannelBufferSize(value int) error {
	if value < MinChannelBufferSize || value > MaxChannelBufferSize {
		return fmt.Errorf("%w: ChannelBufferSize must be between %d and %d, got %d", ErrValueOutOfRange, MinChannelBufferSize, MaxChannelBufferSize, value)
	}
	return nil
}

func (v *Validator) ValidateFeatureFlushThreshold(value int) error {
	if value < MinFeatureFlushThreshold || value > MaxFeatureFlushThreshold {
		return fmt.Errorf("%w: FeatureFlushThreshold must be between %d and %d, got %d", ErrValueOutOfRange, MinFeatureFlushThreshold, MaxFeatureFlushThreshold, value)
	}
	return nil
}

func (v *Validator) ValidateMachineType(value string) error {
	for _, allowed := range AllowedMachineTypes {
		if value == allowed {
			return nil
		}
	}
	return fmt.Errorf("%w: %s is not an allowed machine type", ErrInvalidMachineType, value)
}

func (v *Validator) ValidateTaskCount(value int) error {
	if value < MinTaskCount || value > MaxTaskCount {
		return fmt.Errorf("%w: TaskCount must be between %d and %d, got %d", ErrValueOutOfRange, MinTaskCount, MaxTaskCount, value)
	}
	return nil
}

func (v *Validator) ValidateNodeStatusPropagationDelayMS(value int) error {
	if value < MinNodeStatusPropagationDelayMS || value > MaxNodeStatusPropagationDelayMS {
		return fmt.Errorf("%w: NodeStatusPropagationDelayMS must be between %d and %d, got %d", ErrValueOutOfRange, MinNodeStatusPropagationDelayMS, MaxNodeStatusPropagationDelayMS, value)
	}
	return nil
}

func (v *Validator) ValidateBootDiskType(value string) error {
	for _, allowed := range AllowedBootDiskTypes {
		if value == allowed {
			return nil
		}
	}
	return fmt.Errorf("%w: %s is not an allowed boot disk type (allowed: %v)", ErrInvalidBootDiskType, value, AllowedBootDiskTypes)
}

func (v *Validator) ValidateImageURL(value string) error {
	if value == "" {
		return fmt.Errorf("%w: ImageURL cannot be empty", ErrValidationFailed)
	}
	// Basic validation - should be a valid container registry URL
	if !strings.Contains(value, "/") {
		return fmt.Errorf("%w: ImageURL must be a valid container registry path", ErrValidationFailed)
	}
	return nil
}

func (v *Validator) ValidateBinaryPath(value string) error {
	if value == "" {
		return fmt.Errorf("%w: BinaryPath cannot be empty", ErrValidationFailed)
	}
	return nil
}

func (v *Validator) ValidateAllowedLocations(locations []string) error {
	for _, loc := range locations {
		if !gcpRegionPattern.MatchString(loc) {
			return fmt.Errorf("%w: %s does not match GCP region/zone pattern", ErrInvalidLocation, loc)
		}
	}
	return nil
}

// Cross-field validators

func (v *Validator) ValidateMachineTypeMemoryCompatibility(machineType string, memoryMib int) error {
	// Simplified check - extract machine family and size
	maxMemory := v.getMachineTypeMaxMemory(machineType)
	if maxMemory > 0 && memoryMib > maxMemory {
		return fmt.Errorf("%w: requested %d MiB exceeds machine type %s capacity of ~%d MiB", ErrMemoryExceedsMachine, memoryMib, machineType, maxMemory)
	}
	return nil
}

func (v *Validator) ValidateMachineTypeCPUCompatibility(machineType string, cpuMilli int) error {
	// Simplified check - extract vCPU count from machine type
	maxCPU := v.getMachineTypeMaxCPU(machineType)
	if maxCPU > 0 && cpuMilli > maxCPU*1000 {
		return fmt.Errorf("%w: requested %d milli-CPU exceeds machine type %s capacity of %d vCPUs", ErrCPUExceedsMachine, cpuMilli, machineType, maxCPU)
	}
	return nil
}

// Helper methods to extract machine type capabilities

func (v *Validator) getMachineTypeMaxMemory(machineType string) int {
	// Simplified extraction - map common machine types to memory
	memoryMap := map[string]int{
		"e2-standard-2":  8 * 1024,
		"e2-standard-4":  16 * 1024,
		"e2-standard-8":  32 * 1024,
		"e2-standard-16": 64 * 1024,
		"e2-standard-32": 128 * 1024,
		"n1-standard-1":  3840,
		"n1-standard-2":  7680,
		"n1-standard-4":  15 * 1024,
		"n1-standard-8":  30 * 1024,
		"n1-standard-16": 60 * 1024,
		"n1-standard-32": 120 * 1024,
		// Add more as needed
	}
	if mem, ok := memoryMap[machineType]; ok {
		return mem
	}
	// For highmem variants, multiply by ~2
	if strings.Contains(machineType, "highmem") {
		// Try to extract base and multiply
		for base, mem := range memoryMap {
			baseType := strings.Replace(machineType, "highmem", "standard", 1)
			if base == baseType {
				return mem * 2
			}
		}
	}
	return 0 // Unknown, skip validation
}

func (v *Validator) getMachineTypeMaxCPU(machineType string) int {
	// Extract vCPU count from machine type name
	// Pattern: {family}-{type}-{count}
	parts := strings.Split(machineType, "-")
	if len(parts) < 3 {
		return 0
	}

	// Simple mapping for common machine types
	cpuMap := map[string]int{
		"e2-standard-2":  2,
		"e2-standard-4":  4,
		"e2-standard-8":  8,
		"e2-standard-16": 16,
		"e2-standard-32": 32,
		"n1-standard-1":  1,
		"n1-standard-2":  2,
		"n1-standard-4":  4,
		"n1-standard-8":  8,
		"n1-standard-16": 16,
		"n1-standard-32": 32,
		"n1-standard-64": 64,
		"n1-standard-96": 96,
		// Add more as needed
	}

	if cpu, ok := cpuMap[machineType]; ok {
		return cpu
	}

	return 0 // Unknown, skip validation
}
