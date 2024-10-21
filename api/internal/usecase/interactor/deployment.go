package interactor

import (
	"context"
	"fmt"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/usecasex"
)

type Deployment struct {
	common
	deploymentRepo repo.Deployment
	projectRepo    repo.Project
	workflowRepo   repo.Workflow
	jobRepo        repo.Job
	workspaceRepo  accountrepo.Workspace
	transaction    usecasex.Transaction
	batch          gateway.Batch
	file           gateway.File
}

func NewDeployment(r *repo.Container, gr *gateway.Container) interfaces.Deployment {
	return &Deployment{
		deploymentRepo: r.Deployment,
		projectRepo:    r.Project,
		workflowRepo:   r.Workflow,
		jobRepo:        r.Job,
		workspaceRepo:  r.Workspace,
		transaction:    r.Transaction,
		batch:          gr.Batch,
		file:           gr.File,
	}
}

func (i *Deployment) Fetch(ctx context.Context, ids []id.DeploymentID, operator *usecase.Operator) ([]*deployment.Deployment, error) {
	return i.deploymentRepo.FindByIDs(ctx, ids)
}

func (i *Deployment) FindByWorkspace(ctx context.Context, id accountdomain.WorkspaceID, p *usecasex.Pagination, operator *usecase.Operator) ([]*deployment.Deployment, *usecasex.PageInfo, error) {
	return i.deploymentRepo.FindByWorkspace(ctx, id, p)
}

func (i *Deployment) Create(ctx context.Context, p interfaces.CreateDeploymentParam, operator *usecase.Operator) (result *deployment.Deployment, err error) {
	if err := i.CanWriteWorkspace(p.Workspace, operator); err != nil {
		fmt.Println("HERE0", err.Error())
		return nil, err
	}

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return
	}

	ctx = tx.Context()
	defer func() {
		if err2 := tx.End(ctx); err == nil && err2 != nil {
			err = err2
		}
	}()

	_, err = i.projectRepo.FindByID(ctx, p.Project)
	if err != nil {
		return nil, err
	}

	url, err := i.file.UploadWorkflow(ctx, p.Workflow)
	if err != nil {
		return nil, err
	}

	d, err := deployment.New().
		NewID().
		Project(p.Project).
		Workspace(p.Workspace).
		WorkflowURL(url.String()).
		Version("v0.1"). //version is hardcoded for now @pyshx
		Build()
	if err != nil {
		return nil, err
	}

	if err := i.deploymentRepo.Save(ctx, d); err != nil {
		return nil, err
	}

	tx.Commit()
	return d, nil
}

func (i *Deployment) Execute(ctx context.Context, p interfaces.ExecuteDeploymentParam, operator *usecase.Operator) (_ *job.Job, err error) {
	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return
	}

	ctx = tx.Context()
	defer func() {
		if err2 := tx.End(ctx); err == nil && err2 != nil {
			err = err2
		}
	}()

	d, err := i.deploymentRepo.FindByID(ctx, p.DeploymentID)
	if err != nil {
		return nil, err
	}

	if err := i.CanWriteWorkspace(d.Workspace(), operator); err != nil {
		return nil, err
	}

	j, err := job.New().
		NewID().
		Deployment(d.ID()).
		Workspace(d.Workspace()).
		Status(job.StatusPending).
		Build()
	if err != nil {
		return nil, err
	}

	if err := i.jobRepo.Save(ctx, j); err != nil {
		return nil, err
	}

	_, err = i.batch.SubmitJob(ctx, j.ID(), d.WorkflowUrl(), d.Project())
	if err != nil {
		return nil, interfaces.ErrJobCreationFailed
	}

	tx.Commit()
	return j, nil
}
