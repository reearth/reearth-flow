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
	usecases   interfaces.Container
	Asset      *AssetLoader
	Deployment *DeploymentLoader
	Job        *JobLoader
	Project    *ProjectLoader
	User       *UserLoader
	Workspace  *WorkspaceLoader
}

type DataLoaders struct {
	Asset      AssetDataLoader
	Deployment DeploymentDataLoader
	Job        JobDataLoader
	Project    ProjectDataLoader
	User       UserDataLoader
	Workspace  WorkspaceDataLoader
}

func NewLoaders(usecases *interfaces.Container) *Loaders {
	if usecases == nil {
		return nil
	}
	return &Loaders{
		usecases:   *usecases,
		Asset:      NewAssetLoader(usecases.Asset),
		Deployment: NewDeploymentLoader(usecases.Deployment),
		Job:        NewJobLoader(usecases.Job),
		Project:    NewProjectLoader(usecases.Project),
		User:       NewUserLoader(usecases.User),
		Workspace:  NewWorkspaceLoader(usecases.Workspace),
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
		Asset:      l.Asset.DataLoader(ctx),
		Deployment: l.Deployment.DataLoader(ctx),
		Job:        l.Job.DataLoader(ctx),
		Project:    l.Project.DataLoader(ctx),
		User:       l.User.DataLoader(ctx),
		Workspace:  l.Workspace.DataLoader(ctx),
	}
}

func (l Loaders) OrdinaryDataLoaders(ctx context.Context) *DataLoaders {
	return &DataLoaders{
		Asset:      l.Asset.OrdinaryDataLoader(ctx),
		Deployment: l.Deployment.OrdinaryDataLoader(ctx),
		Job:        l.Job.OrdinaryDataLoader(ctx),
		Project:    l.Project.OrdinaryDataLoader(ctx),
		User:       l.User.OrdinaryDataLoader(ctx),
		Workspace:  l.Workspace.OrdinaryDataLoader(ctx),
	}
}
