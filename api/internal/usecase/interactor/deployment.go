package interactor

import (
	"context"
	"net/url"

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

func (i *Deployment) Create(ctx context.Context, dp interfaces.CreateDeploymentParam, operator *usecase.Operator) (result *deployment.Deployment, err error) {
	// if err := i.CanWriteWorkspace(dp.Workspace, operator); err != nil {
	// 	return nil, err
	// }

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

	_, err = i.projectRepo.FindByID(ctx, dp.Project)
	if err != nil {
		return nil, err
	}

	url, err := i.file.UploadWorkflow(ctx, dp.Workflow)
	if err != nil {
		return nil, err
	}

	d := deployment.New().
		NewID().
		Project(dp.Project).
		Workspace(dp.Workspace).
		WorkflowURL(url.String()).
		Version("v0.1") //version is hardcoded for now @pyshx
	if dp.Description != nil {
		d = d.Description(*dp.Description)
	}

	dep, err := d.Build()
	if err != nil {
		return nil, err
	}

	if err := i.deploymentRepo.Save(ctx, dep); err != nil {
		return nil, err
	}

	tx.Commit()
	return dep, nil
}

func (i *Deployment) Update(ctx context.Context, dp interfaces.UpdateDeploymentParam, operator *usecase.Operator) (_ *deployment.Deployment, err error) {
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

	d, err := i.deploymentRepo.FindByID(ctx, dp.ID)
	if err != nil {
		return nil, err
	}
	if err := i.CanWriteWorkspace(d.Workspace(), operator); err != nil {
		return nil, err
	}

	if dp.Workflow != nil {
		if url, _ := url.Parse(d.WorkflowUrl()); url != nil {
			if err := i.file.RemoveWorkflow(ctx, url); err != nil {
				return nil, err
			}
		}

		url, err := i.file.UploadWorkflow(ctx, dp.Workflow)
		if err != nil {
			return nil, err
		}
		d.SetWorkflowUrl(url.String())
	}

	if dp.Description != nil {
		d.SetDescription(*dp.Description)
	}

	// d.SetVersion() // version is hardcoded for now but will need to be incremented here eventually

	if err := i.deploymentRepo.Save(ctx, d); err != nil {
		return nil, err
	}

	tx.Commit()
	return d, nil
}

func (i *Deployment) Delete(ctx context.Context, deploymentID id.DeploymentID, operator *usecase.Operator) (err error) {
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

	dep, err := i.deploymentRepo.FindByID(ctx, deploymentID)
	if err != nil {
		return err
	}
	if err := i.CanWriteWorkspace(dep.Workspace(), operator); err != nil {
		return err
	}

	if url, _ := url.Parse(dep.WorkflowUrl()); url != nil {
		if err := i.file.RemoveWorkflow(ctx, url); err != nil {
			return err
		}
	}

	if err := i.deploymentRepo.Remove(ctx, deploymentID); err != nil {
		return err
	}

	tx.Commit()
	return nil
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
