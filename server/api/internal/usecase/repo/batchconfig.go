package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/batchconfig"
)

// BatchConfig repository provides persistence operations for batch configurations
type BatchConfig interface {
	// FindByID retrieves a batch configuration by its ID
	FindByID(ctx context.Context, id batchconfig.ID) (*batchconfig.BatchConfig, error)

	// FindByWorkspaceID retrieves a batch configuration for a specific workspace
	// Returns nil if no custom configuration exists for the workspace (use defaults)
	FindByWorkspaceID(ctx context.Context, workspaceID batchconfig.WorkspaceID) (*batchconfig.BatchConfig, error)

	// Save creates or updates a batch configuration
	Save(ctx context.Context, config *batchconfig.BatchConfig) error

	// Remove deletes a batch configuration (reverts to environment defaults)
	Remove(ctx context.Context, id batchconfig.ID) error

	// RemoveByWorkspaceID deletes a workspace's batch configuration
	RemoveByWorkspaceID(ctx context.Context, workspaceID batchconfig.WorkspaceID) error
}
