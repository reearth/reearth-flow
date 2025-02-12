package interactor

import (
	"context"
	"fmt"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/usecasex"
)

type Trigger struct {
	common
	triggerRepo    repo.Trigger
	deploymentRepo repo.Deployment
	jobRepo        repo.Job
	workspaceRepo  accountrepo.Workspace
	transaction    usecasex.Transaction
	batch          gateway.Batch
	file           gateway.File
	job            interfaces.Job
}

func NewTrigger(r *repo.Container, gr *gateway.Container, jobUsecase interfaces.Job) interfaces.Trigger {
	return &Trigger{
		triggerRepo:    r.Trigger,
		deploymentRepo: r.Deployment,
		jobRepo:        r.Job,
		workspaceRepo:  r.Workspace,
		transaction:    r.Transaction,
		batch:          gr.Batch,
		file:           gr.File,
		job:            jobUsecase,
	}
}

func (i *Trigger) Fetch(ctx context.Context, ids []id.TriggerID, operator *usecase.Operator) ([]*trigger.Trigger, error) {
	return i.triggerRepo.FindByIDs(ctx, ids)
}

func (i *Trigger) FindByWorkspace(ctx context.Context, id accountdomain.WorkspaceID, p *interfaces.PaginationParam, operator *usecase.Operator) ([]*trigger.Trigger, *interfaces.PageBasedInfo, error) {
	if err := i.CanReadWorkspace(id, operator); err != nil {
		return nil, nil, err
	}
	return i.triggerRepo.FindByWorkspace(ctx, id, p)
}

func (i *Trigger) FindByID(ctx context.Context, id id.TriggerID, operator *usecase.Operator) (*trigger.Trigger, error) {
	return i.triggerRepo.FindByID(ctx, id)
}

func (i *Trigger) Create(ctx context.Context, param interfaces.CreateTriggerParam, operator *usecase.Operator) (result *trigger.Trigger, err error) {
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

	if _, err = i.deploymentRepo.FindByID(ctx, param.DeploymentID); err != nil {
		return nil, err
	}

	t := trigger.New().
		NewID().
		Workspace(param.WorkspaceID).
		Deployment(param.DeploymentID).
		Description(param.Description).
		EventSource(param.EventSource).
		UpdatedAt(time.Now())

	if param.EventSource == "TIME_DRIVEN" {
		t = t.TimeInterval(trigger.TimeInterval(param.TimeInterval))
	} else if param.EventSource == "API_DRIVEN" {
		t = t.AuthToken(param.AuthToken)
	}

	trg, err := t.Build()
	if err != nil {
		return nil, err
	}

	if err := i.triggerRepo.Save(ctx, trg); err != nil {
		return nil, err
	}

	tx.Commit()
	return trg, nil
}

func (i *Trigger) ExecuteAPITrigger(ctx context.Context, p interfaces.ExecuteAPITriggerParam, operator *usecase.Operator) (_ *job.Job, err error) {
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

	trigger, err := i.triggerRepo.FindByID(ctx, p.TriggerID)
	if err != nil {
		return nil, err
	}

	if trigger.EventSource() == "API_DRIVEN" {
		if p.AuthenticationToken != *trigger.AuthToken() {
			return nil, fmt.Errorf("invalid auth token")
		}
	}

	deployment, err := i.deploymentRepo.FindByID(ctx, trigger.Deployment())
	if err != nil {
		return nil, err
	}

	j, err := job.New().
		NewID().
		Deployment(deployment.ID()).
		Workspace(deployment.Workspace()).
		Status(job.StatusPending).
		StartedAt(time.Now()).
		Build()
	if err != nil {
		return nil, err
	}

	metadataURL, err := i.file.UploadMetadata(ctx, j.ID().String(), []string{})
	j.SetMetadataURL(metadataURL.String())
	if err := i.jobRepo.Save(ctx, j); err != nil {
		return nil, err
	}

	var projectID id.ProjectID
	if deployment.Project() != nil {
		projectID = *deployment.Project()
	}

	gcpJobID, err := i.batch.SubmitJob(ctx, j.ID(), deployment.WorkflowURL(), j.MetadataURL(), p.Variables, projectID, deployment.Workspace())
	if err != nil {
		return nil, interfaces.ErrJobCreationFailed
	}

	j.SetGCPJobID(gcpJobID)

	if err := i.job.StartMonitoring(ctx, j, p.NotificationURL, operator); err != nil {
		return nil, err
	}

	tx.Commit()

	return j, nil
}

func (i *Trigger) Update(ctx context.Context, param interfaces.UpdateTriggerParam, operator *usecase.Operator) (_ *trigger.Trigger, err error) {
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

	t, err := i.triggerRepo.FindByID(ctx, param.ID)
	if err != nil {
		return nil, err
	}

	if param.DeploymentID != nil {
		if _, err = i.deploymentRepo.FindByID(ctx, *param.DeploymentID); err != nil {
			return nil, err
		}
		t.SetDeployment(*param.DeploymentID)
	}

	if param.Description != nil {
		t.SetDescription(*param.Description)
	}

	if param.EventSource == "TIME_DRIVEN" {
		t.SetEventSource(trigger.EventSourceType(param.EventSource))
		t.SetTimeInterval(trigger.TimeInterval(param.TimeInterval))
		t.SetAuthToken("")
	} else if param.EventSource == "API_DRIVEN" {
		t.SetEventSource(trigger.EventSourceType(param.EventSource))
		t.SetTimeInterval("")
		t.SetAuthToken(param.AuthToken)
	}

	if err := i.triggerRepo.Save(ctx, t); err != nil {
		return nil, err
	}

	tx.Commit()
	return t, nil
}

func (i *Trigger) Delete(ctx context.Context, id id.TriggerID, operator *usecase.Operator) (err error) {
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

	if err := i.triggerRepo.Remove(ctx, id); err != nil {
		return err
	}

	tx.Commit()
	return nil
}
