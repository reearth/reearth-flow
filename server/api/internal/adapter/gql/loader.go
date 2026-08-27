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
	usecases     interfaces.Container
	Asset        *AssetLoader
	Deployment   *DeploymentLoader
	Job          *JobLoader
	Log          *LogLoader
	Parameter    *ParameterLoader
	Project      *ProjectLoader
	Trigger      *TriggerLoader
	User         *UserLoader
	Workspace    *WorkspaceLoader
	WorkerConfig *WorkerConfigLoader
}

type DataLoaders struct {
	Asset               AssetDataLoader
	Deployment          DeploymentDataLoader
	DeploymentByProject DeploymentByProjectDataLoader
	Job                 JobDataLoader
	ParametersByProject ParametersByProjectDataLoader
	LogsByJob           LogsByJobDataLoader
	Parameter           ParameterDataLoader
	Project             ProjectDataLoader
	Trigger             TriggerDataLoader
	User                UserDataLoader
	Workspace           WorkspaceDataLoader
	WorkerConfig        WorkerConfigDataLoader
}

func NewLoaders(usecases *interfaces.Container) *Loaders {
	if usecases == nil {
		return nil
	}
	return &Loaders{
		usecases:     *usecases,
		Asset:        NewAssetLoader(usecases.Asset),
		Deployment:   NewDeploymentLoader(usecases.Deployment),
		Job:          NewJobLoader(usecases.Job),
		Log:          NewLogLoader(usecases.Log),
		Parameter:    NewParameterLoader(usecases.Parameter),
		Project:      NewProjectLoader(usecases.Project),
		Trigger:      NewTriggerLoader(usecases.Trigger),
		User:         NewUserLoader(usecases.User),
		Workspace:    NewWorkspaceLoader(usecases.Workspace),
		WorkerConfig: NewWorkerConfigLoader(usecases.WorkerConfig),
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
		Asset:               l.Asset.DataLoader(ctx),
		Deployment:          l.Deployment.DataLoader(ctx),
		DeploymentByProject: l.Deployment.ByProjectDataLoader(ctx),
		Job:                 l.Job.DataLoader(ctx),
		ParametersByProject: l.Parameter.ByProjectDataLoader(ctx),
		LogsByJob:           l.Log.ByJobDataLoader(ctx),
		Parameter:           l.Parameter.DataLoader(ctx),
		Project:             l.Project.DataLoader(ctx),
		Trigger:             l.Trigger.DataLoader(ctx),
		User:                l.User.DataLoader(ctx),
		Workspace:           l.Workspace.DataLoader(ctx),
		WorkerConfig:        l.WorkerConfig.DataLoader(ctx),
	}
}

func (l Loaders) OrdinaryDataLoaders(ctx context.Context) *DataLoaders {
	return &DataLoaders{
		Asset:               l.Asset.OrdinaryDataLoader(ctx),
		Deployment:          l.Deployment.OrdinaryDataLoader(ctx),
		DeploymentByProject: l.Deployment.OrdinaryByProjectDataLoader(ctx),
		Job:                 l.Job.OrdinaryDataLoader(ctx),
		ParametersByProject: l.Parameter.OrdinaryByProjectDataLoader(ctx),
		LogsByJob:           l.Log.OrdinaryByJobDataLoader(ctx),
		Parameter:           l.Parameter.OrdinaryDataLoader(ctx),
		Project:             l.Project.OrdinaryDataLoader(ctx),
		Trigger:             l.Trigger.OrdinaryDataLoader(ctx),
		User:                l.User.OrdinaryDataLoader(ctx),
		Workspace:           l.Workspace.OrdinaryDataLoader(ctx),
		WorkerConfig:        l.WorkerConfig.OrdinaryDataLoader(ctx),
	}
}
