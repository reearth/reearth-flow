package gql

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
)

type jobResolver struct{ *Resolver }

func (r *jobResolver) Deployment(ctx context.Context, obj *gqlmodel.Job) (*gqlmodel.Deployment, error) {
	if obj.DeploymentID == nil {
		return nil, nil
	}
	return dataloaders(ctx).Deployment.Load(*obj.DeploymentID)
}

func (r *jobResolver) Workspace(ctx context.Context, obj *gqlmodel.Job) (*gqlmodel.Workspace, error) {
	return dataloaders(ctx).Workspace.Load(obj.WorkspaceID)
}

func (r *jobResolver) Logs(ctx context.Context, obj *gqlmodel.Job, since time.Time) ([]*gqlmodel.Log, error) {
	return loaders(ctx).Log.GetLogs(ctx, since, obj.ID)
}

// FailedNodes reads the terminal per-node failure rows persisted at
// job-completion merge time (interactor/job.go's persistTerminalDiagnostics,
// Mongo-only — never written to Redis, see loader_diagnostic.go).
func (r *jobResolver) FailedNodes(ctx context.Context, obj *gqlmodel.Job) ([]*gqlmodel.Diagnostic, error) {
	return loaders(ctx).Diagnostic.GetFailedNodes(ctx, obj.ID)
}

// DroppedEventCount reads the single per-job summary row persisted alongside
// FailedNodes (see loader_diagnostic.go).
func (r *jobResolver) DroppedEventCount(ctx context.Context, obj *gqlmodel.Job) (*int, error) {
	return loaders(ctx).Diagnostic.GetDroppedEventCount(ctx, obj.ID)
}
