package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/pkg/batchconfig"
	"github.com/reearth/reearthx/account/accountdomain"
)

// BatchConfig usecase interface
type BatchConfig interface {
	GetBatchConfig(ctx context.Context, param GetBatchConfigParam) (*batchconfig.BatchConfig, error)
	GetEffectiveBatchConfig(ctx context.Context, param GetEffectiveBatchConfigParam) (*EffectiveBatchConfig, error)
	UpdateBatchConfig(ctx context.Context, param UpdateBatchConfigParam) (*batchconfig.BatchConfig, []BatchConfigValidationError, error)
	ResetBatchConfig(ctx context.Context, param ResetBatchConfigParam) error
	ValidateBatchConfig(ctx context.Context, param ValidateBatchConfigParam) (bool, []BatchConfigValidationError, error)
	GetBatchConfigConstraints(ctx context.Context) (*BatchConfigConstraints, error)
}

// BatchConfig parameters and types

type GetBatchConfigParam struct {
	WorkspaceID accountdomain.WorkspaceID
	Operator    *usecase.Operator
}

type GetEffectiveBatchConfigParam struct {
	WorkspaceID accountdomain.WorkspaceID
	Operator    *usecase.Operator
}

type UpdateBatchConfigParam struct {
	WorkspaceID                  accountdomain.WorkspaceID
	Operator                     *usecase.Operator
	ComputeCpuMilli              *int
	ComputeMemoryMib             *int
	BootDiskSizeGB               *int
	MaxConcurrency               *int
	ThreadPoolSize               *int
	ChannelBufferSize            *int
	FeatureFlushThreshold        *int
	MachineType                  *string
	TaskCount                    *int
	NodeStatusPropagationDelayMS *int
	BootDiskType                 *string
	ImageURL                     *string
	BinaryPath                   *string
	AllowedLocations             []string
}

type ResetBatchConfigParam struct {
	WorkspaceID accountdomain.WorkspaceID
	Operator    *usecase.Operator
}

type ValidateBatchConfigParam struct {
	WorkspaceID                  accountdomain.WorkspaceID
	Operator                     *usecase.Operator
	ComputeCpuMilli              *int
	ComputeMemoryMib             *int
	BootDiskSizeGB               *int
	MaxConcurrency               *int
	ThreadPoolSize               *int
	ChannelBufferSize            *int
	FeatureFlushThreshold        *int
	MachineType                  *string
	TaskCount                    *int
	NodeStatusPropagationDelayMS *int
	BootDiskType                 *string
	ImageURL                     *string
	BinaryPath                   *string
	AllowedLocations             []string
}

type EffectiveBatchConfig struct {
	WorkspaceID                  accountdomain.WorkspaceID
	ComputeCpuMilli              int
	ComputeMemoryMib             int
	BootDiskSizeGB               int
	MaxConcurrency               int
	ThreadPoolSize               int
	ChannelBufferSize            int
	FeatureFlushThreshold        int
	MachineType                  string
	TaskCount                    int
	NodeStatusPropagationDelayMS int
	BootDiskType                 string
	ImageURL                     string
	BinaryPath                   string
	AllowedLocations             []string
	HasCustomConfig              bool
	CustomConfigID               *batchconfig.ID
}

type BatchConfigValidationError struct {
	Field   string
	Message string
}

type BatchConfigConstraints struct {
	ComputeCpuMilliMin              int
	ComputeCpuMilliMax              int
	ComputeMemoryMibMin             int
	ComputeMemoryMibMax             int
	BootDiskSizeGBMin               int
	BootDiskSizeGBMax               int
	MaxConcurrencyMin               int
	MaxConcurrencyMax               int
	ThreadPoolSizeMin               int
	ThreadPoolSizeMax               int
	ChannelBufferSizeMin            int
	ChannelBufferSizeMax            int
	FeatureFlushThresholdMin        int
	FeatureFlushThresholdMax        int
	TaskCountMin                    int
	TaskCountMax                    int
	NodeStatusPropagationDelayMSMin int
	NodeStatusPropagationDelayMSMax int
	AllowedMachineTypes             []string
	AllowedBootDiskTypes            []string
}
