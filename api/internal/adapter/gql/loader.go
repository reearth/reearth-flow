package gql

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
)

const (
	dataLoaderWait     = 1 * time.Millisecond
	dataLoaderMaxBatch = 100
)

type Loaders struct {
	usecases  interfaces.Container
	Asset     *AssetLoader
	Project   *ProjectLoader
	Workspace *WorkspaceLoader
	User      *UserLoader
}

type DataLoaders struct {
	Asset     AssetDataLoader
	Project   ProjectDataLoader
	Workspace WorkspaceDataLoader
	User      UserDataLoader
}

func NewLoaders(usecases *interfaces.Container) *Loaders {
	if usecases == nil {
		return nil
	}
	return &Loaders{
		usecases:  *usecases,
		Asset:     NewAssetLoader(usecases.Asset),
		Project:   NewProjectLoader(usecases.Project),
		Workspace: NewWorkspaceLoader(usecases.Workspace),
		User:      NewUserLoader(usecases.User),
	}
}

func (l Loaders) DataLoadersWith(ctx context.Context, enabled bool) *DataLoaders {
	if enabled {
		return l.DataLoaders(ctx)
	}
	return l.OrdinaryDataLoaders(ctx)
}

func (l Loaders) DataLoaders(ctx context.Context) *DataLoaders {
	return &DataLoaders{
		Asset:     l.Asset.DataLoader(ctx),
		Project:   l.Project.DataLoader(ctx),
		Workspace: l.Workspace.DataLoader(ctx),
		User:      l.User.DataLoader(ctx),
	}
}

func (l Loaders) OrdinaryDataLoaders(ctx context.Context) *DataLoaders {
	return &DataLoaders{
		Asset:     l.Asset.OrdinaryDataLoader(ctx),
		Project:   l.Project.OrdinaryDataLoader(ctx),
		Workspace: l.Workspace.OrdinaryDataLoader(ctx),
		User:      l.User.OrdinaryDataLoader(ctx),
	}
}
