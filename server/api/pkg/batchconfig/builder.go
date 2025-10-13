package batchconfig

import (
	"time"
)

// Builder provides a fluent interface for constructing BatchConfig instances
type Builder struct {
	b *BatchConfig
}

// New creates a new BatchConfig builder
func New() *Builder {
	return &Builder{
		b: &BatchConfig{
			changeHistory: make([]ConfigChange, 0),
		},
	}
}

// ID sets the batch config ID
func (bb *Builder) ID(id ID) *Builder {
	bb.b.id = id
	return bb
}

// NewID generates and sets a new ID
func (bb *Builder) NewID() *Builder {
	bb.b.id = NewID()
	return bb
}

// WorkspaceID sets the workspace ID
func (bb *Builder) WorkspaceID(workspaceID WorkspaceID) *Builder {
	bb.b.workspaceID = workspaceID
	return bb
}

// CreatedAt sets the creation timestamp
func (bb *Builder) CreatedAt(createdAt time.Time) *Builder {
	bb.b.createdAt = createdAt
	return bb
}

// UpdatedAt sets the update timestamp
func (bb *Builder) UpdatedAt(updatedAt time.Time) *Builder {
	bb.b.updatedAt = updatedAt
	return bb
}

// CreatedBy sets the creator user ID
func (bb *Builder) CreatedBy(createdBy string) *Builder {
	bb.b.createdBy = createdBy
	return bb
}

// UpdatedBy sets the last updater user ID
func (bb *Builder) UpdatedBy(updatedBy string) *Builder {
	bb.b.updatedBy = updatedBy
	return bb
}

// Tier A Parameters
func (bb *Builder) ComputeCpuMilli(value *int) *Builder {
	bb.b.computeCpuMilli = value
	return bb
}

func (bb *Builder) ComputeMemoryMib(value *int) *Builder {
	bb.b.computeMemoryMib = value
	return bb
}

func (bb *Builder) BootDiskSizeGB(value *int) *Builder {
	bb.b.bootDiskSizeGB = value
	return bb
}

func (bb *Builder) MaxConcurrency(value *int) *Builder {
	bb.b.maxConcurrency = value
	return bb
}

func (bb *Builder) ThreadPoolSize(value *int) *Builder {
	bb.b.threadPoolSize = value
	return bb
}

func (bb *Builder) ChannelBufferSize(value *int) *Builder {
	bb.b.channelBufferSize = value
	return bb
}

func (bb *Builder) FeatureFlushThreshold(value *int) *Builder {
	bb.b.featureFlushThreshold = value
	return bb
}

// Tier B Parameters
func (bb *Builder) MachineType(value *string) *Builder {
	bb.b.machineType = value
	return bb
}

func (bb *Builder) TaskCount(value *int) *Builder {
	bb.b.taskCount = value
	return bb
}

func (bb *Builder) NodeStatusPropagationDelayMS(value *int) *Builder {
	bb.b.nodeStatusPropagationDelayMS = value
	return bb
}

// Tier C Parameters
func (bb *Builder) BootDiskType(value *string) *Builder {
	bb.b.bootDiskType = value
	return bb
}

func (bb *Builder) ImageURL(value *string) *Builder {
	bb.b.imageURL = value
	return bb
}

func (bb *Builder) BinaryPath(value *string) *Builder {
	bb.b.binaryPath = value
	return bb
}

func (bb *Builder) AllowedLocations(value []string) *Builder {
	bb.b.allowedLocations = value
	return bb
}

// ChangeHistory sets the change history
func (bb *Builder) ChangeHistory(history []ConfigChange) *Builder {
	bb.b.changeHistory = history
	return bb
}

// Build creates the BatchConfig instance
func (bb *Builder) Build() (*BatchConfig, error) {
	if bb.b.id.IsNil() {
		return nil, ErrInvalidID
	}
	if bb.b.workspaceID.IsNil() {
		return nil, ErrInvalidID
	}
	if bb.b.createdAt.IsZero() {
		bb.b.createdAt = time.Now()
	}
	if bb.b.updatedAt.IsZero() {
		bb.b.updatedAt = bb.b.createdAt
	}
	if bb.b.changeHistory == nil {
		bb.b.changeHistory = make([]ConfigChange, 0)
	}
	return bb.b, nil
}

// MustBuild creates the BatchConfig instance or panics
func (bb *Builder) MustBuild() *BatchConfig {
	b, err := bb.Build()
	if err != nil {
		panic(err)
	}
	return b
}
