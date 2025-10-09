package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/batchconfig"
	"github.com/reearth/reearthx/account/accountdomain"
)

type BatchConfigDocument struct {
	ID          string    `bson:"id"`
	WorkspaceID string    `bson:"workspaceid"`
	CreatedAt   time.Time `bson:"createdat"`
	UpdatedAt   time.Time `bson:"updatedat"`
	CreatedBy   string    `bson:"createdby"`
	UpdatedBy   string    `bson:"updatedby"`

	// Tier A parameters
	ComputeCpuMilli       *int `bson:"computecpumilli,omitempty"`
	ComputeMemoryMib      *int `bson:"computememorymib,omitempty"`
	BootDiskSizeGB        *int `bson:"bootdisksizegb,omitempty"`
	MaxConcurrency        *int `bson:"maxconcurrency,omitempty"`
	ThreadPoolSize        *int `bson:"threadpoolsize,omitempty"`
	ChannelBufferSize     *int `bson:"channelbuffersize,omitempty"`
	FeatureFlushThreshold *int `bson:"featureflushhreshold,omitempty"`

	// Tier B parameters
	MachineType                  *string `bson:"machinetype,omitempty"`
	TaskCount                    *int    `bson:"taskcount,omitempty"`
	NodeStatusPropagationDelayMS *int    `bson:"nodestatuspropagationdelayms,omitempty"`

	// Tier C parameters
	BootDiskType     *string  `bson:"bootdisktype,omitempty"`
	ImageURL         *string  `bson:"imageurl,omitempty"`
	BinaryPath       *string  `bson:"binarypath,omitempty"`
	AllowedLocations []string `bson:"allowedlocations,omitempty"`

	// Audit trail
	ChangeHistory []ConfigChangeDocument `bson:"changehistory,omitempty"`
}

type ConfigChangeDocument struct {
	Timestamp time.Time   `bson:"timestamp"`
	ChangedBy string      `bson:"changedby"`
	FieldName string      `bson:"fieldname"`
	OldValue  interface{} `bson:"oldvalue,omitempty"`
	NewValue  interface{} `bson:"newvalue,omitempty"`
}

// NewBatchConfig converts domain model to MongoDB document
func NewBatchConfig(config *batchconfig.BatchConfig) (*BatchConfigDocument, error) {
	if config == nil {
		return nil, nil
	}

	doc := &BatchConfigDocument{
		ID:          config.ID().String(),
		WorkspaceID: config.WorkspaceID().String(),
		CreatedAt:   config.CreatedAt(),
		UpdatedAt:   config.UpdatedAt(),
		CreatedBy:   config.CreatedBy(),
		UpdatedBy:   config.UpdatedBy(),

		// Tier A
		ComputeCpuMilli:       config.ComputeCpuMilli(),
		ComputeMemoryMib:      config.ComputeMemoryMib(),
		BootDiskSizeGB:        config.BootDiskSizeGB(),
		MaxConcurrency:        config.MaxConcurrency(),
		ThreadPoolSize:        config.ThreadPoolSize(),
		ChannelBufferSize:     config.ChannelBufferSize(),
		FeatureFlushThreshold: config.FeatureFlushThreshold(),

		// Tier B
		MachineType:                  config.MachineType(),
		TaskCount:                    config.TaskCount(),
		NodeStatusPropagationDelayMS: config.NodeStatusPropagationDelayMS(),

		// Tier C
		BootDiskType:     config.BootDiskType(),
		ImageURL:         config.ImageURL(),
		BinaryPath:       config.BinaryPath(),
		AllowedLocations: config.AllowedLocations(),
	}

	// Convert change history
	history := config.ChangeHistory()
	if len(history) > 0 {
		doc.ChangeHistory = make([]ConfigChangeDocument, len(history))
		for i, change := range history {
			doc.ChangeHistory[i] = ConfigChangeDocument{
				Timestamp: change.Timestamp,
				ChangedBy: change.ChangedBy,
				FieldName: change.FieldName,
				OldValue:  change.OldValue,
				NewValue:  change.NewValue,
			}
		}
	}

	return doc, nil
}

// Model converts MongoDB document to domain model
func (d *BatchConfigDocument) Model() (*batchconfig.BatchConfig, error) {
	if d == nil {
		return nil, nil
	}

	id, err := batchconfig.IDFrom(d.ID)
	if err != nil {
		return nil, err
	}

	workspaceID, err := accountdomain.WorkspaceIDFrom(d.WorkspaceID)
	if err != nil {
		return nil, err
	}

	builder := batchconfig.New().
		ID(id).
		WorkspaceID(workspaceID).
		CreatedAt(d.CreatedAt).
		UpdatedAt(d.UpdatedAt).
		CreatedBy(d.CreatedBy).
		UpdatedBy(d.UpdatedBy).
		// Tier A
		ComputeCpuMilli(d.ComputeCpuMilli).
		ComputeMemoryMib(d.ComputeMemoryMib).
		BootDiskSizeGB(d.BootDiskSizeGB).
		MaxConcurrency(d.MaxConcurrency).
		ThreadPoolSize(d.ThreadPoolSize).
		ChannelBufferSize(d.ChannelBufferSize).
		FeatureFlushThreshold(d.FeatureFlushThreshold).
		// Tier B
		MachineType(d.MachineType).
		TaskCount(d.TaskCount).
		NodeStatusPropagationDelayMS(d.NodeStatusPropagationDelayMS).
		// Tier C
		BootDiskType(d.BootDiskType).
		ImageURL(d.ImageURL).
		BinaryPath(d.BinaryPath).
		AllowedLocations(d.AllowedLocations)

	// Convert change history
	if len(d.ChangeHistory) > 0 {
		history := make([]batchconfig.ConfigChange, len(d.ChangeHistory))
		for i, change := range d.ChangeHistory {
			history[i] = batchconfig.ConfigChange{
				Timestamp: change.Timestamp,
				ChangedBy: change.ChangedBy,
				FieldName: change.FieldName,
				OldValue:  change.OldValue,
				NewValue:  change.NewValue,
			}
		}
		builder = builder.ChangeHistory(history)
	}

	return builder.Build()
}
