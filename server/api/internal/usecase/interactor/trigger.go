package interactor

import (
	"context"
	"fmt"
	"time"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/usecasex"
)

type Trigger struct {
	triggerRepo       repo.Trigger
	deploymentRepo    repo.Deployment
	jobRepo           repo.Job
	workerConfigRepo  repo.WorkerConfig
	transaction       usecasex.Transaction
	batch             gateway.Batch
	file              gateway.File
	job               interfaces.Job
	scheduler         gateway.Scheduler
	permissionChecker gateway.PermissionChecker
}

func NewTrigger(r *repo.Container, gr *gateway.Container, jobUsecase interfaces.Job, permissionChecker gateway.PermissionChecker) interfaces.Trigger {
	return &Trigger{
		triggerRepo:       r.Trigger,
		deploymentRepo:    r.Deployment,
		jobRepo:           r.Job,
		workerConfigRepo:  r.WorkerConfig,
		transaction:       r.Transaction,
		batch:             gr.Batch,
		file:              gr.File,
		job:               jobUsecase,
		scheduler:         gr.Scheduler,
		permissionChecker: permissionChecker,
	}
}

func (i *Trigger) checkPermission(ctx context.Context, action string) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceTrigger, action)
}

func (i *Trigger) Fetch(ctx context.Context, ids []id.TriggerID) ([]*trigger.Trigger, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	return i.triggerRepo.FindByIDs(ctx, ids)
}

func (i *Trigger) FindByWorkspace(ctx context.Context, id id.WorkspaceID, p *interfaces.PaginationParam) ([]*trigger.Trigger, *interfaces.PageBasedInfo, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, nil, err
	}

	return i.triggerRepo.FindByWorkspace(ctx, id, p)
}

func (i *Trigger) FindByID(ctx context.Context, id id.TriggerID) (*trigger.Trigger, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	return i.triggerRepo.FindByID(ctx, id)
}

func (i *Trigger) Create(ctx context.Context, param interfaces.CreateTriggerParam) (result *trigger.Trigger, err error) {
	if err := i.checkPermission(ctx, rbac.ActionCreate); err != nil {
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

	if trg.EventSource() == "TIME_DRIVEN" && i.scheduler != nil {
		if err := i.scheduler.CreateScheduledJob(ctx, trg); err != nil {
			log.Errorf("Failed to create scheduled job for trigger %s: %v", trg.ID(), err)
			// Don't fail trigger creation if scheduler registration fails
			// The trigger can still be executed manually @pyshx
		}
	}

	tx.Commit()
	return trg, nil
}

func (i *Trigger) ExecuteAPITrigger(ctx context.Context, p interfaces.ExecuteAPITriggerParam) (_ *job.Job, err error) {
	if err := i.checkPermission(ctx, rbac.ActionCreate); err != nil {
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
	if err != nil {
		return nil, err
	}
	j.SetMetadataURL(metadataURL.String())
	if err := i.jobRepo.Save(ctx, j); err != nil {
		return nil, err
	}

	var projectID id.ProjectID
	if deployment.Project() != nil {
		projectID = *deployment.Project()
	}

	// Fetch workspace-specific worker config if available
	workerCfg, err := i.workerConfigRepo.FindByWorkspace(ctx, deployment.Workspace())
	if err != nil {
		log.Debugfc(ctx, "[Trigger] Failed to fetch worker config: %v\n", err)
		// Continue with nil config to use environment defaults
	}

	gcpJobID, err := i.batch.SubmitJob(ctx, j.ID(), deployment.WorkflowURL(), j.MetadataURL(), p.Variables, projectID, deployment.Workspace(), workerCfg)
	if err != nil {
		log.Debugfc(ctx, "[Trigger] Job submission failed: %v\n", err)
		return nil, interfaces.ErrJobCreationFailed
	}

	j.SetGCPJobID(gcpJobID)
	if err := i.jobRepo.Save(ctx, j); err != nil {
		log.Errorf("Failed to save job %s with GCP ID: %v", j.ID(), err)
		return nil, err
	}

	if err := i.job.StartMonitoring(ctx, j, p.NotificationURL); err != nil {
		log.Errorf("Failed to start monitoring for job %s: %v", j.ID(), err)
		return nil, err
	}

	tx.Commit()
	return j, nil
}

func (i *Trigger) ExecuteTimeDrivenTrigger(ctx context.Context, p interfaces.ExecuteTimeDrivenTriggerParam) (_ *job.Job, err error) {
	if err := i.checkPermission(ctx, rbac.ActionCreate); err != nil {
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

	trigger, err := i.triggerRepo.FindByID(ctx, p.TriggerID)
	if err != nil {
		return nil, err
	}

	if trigger.EventSource() != "TIME_DRIVEN" {
		return nil, fmt.Errorf("trigger is not time-driven")
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
	if err != nil {
		return nil, err
	}
	j.SetMetadataURL(metadataURL.String())
	if err := i.jobRepo.Save(ctx, j); err != nil {
		return nil, err
	}

	var projectID id.ProjectID
	if deployment.Project() != nil {
		projectID = *deployment.Project()
	}

	// Use empty variables for time-driven triggers
	variables := make(map[string]interface{})

	// Fetch workspace-specific worker config if available
	workerCfg, err := i.workerConfigRepo.FindByWorkspace(ctx, deployment.Workspace())
	if err != nil {
		log.Debugfc(ctx, "[Trigger] Failed to fetch worker config: %v\n", err)
		// Continue with nil config to use environment defaults
	}

	gcpJobID, err := i.batch.SubmitJob(ctx, j.ID(), deployment.WorkflowURL(), j.MetadataURL(), variables, projectID, deployment.Workspace(), workerCfg)
	if err != nil {
		log.Debugfc(ctx, "[Trigger] Time-driven job submission failed: %v\n", err)
		return nil, interfaces.ErrJobCreationFailed
	}

	j.SetGCPJobID(gcpJobID)
	if err := i.jobRepo.Save(ctx, j); err != nil {
		log.Errorf("Failed to save time-driven job %s with GCP ID: %v", j.ID(), err)
		return nil, err
	}

	// Update last triggered time
	trigger.SetLastTriggered(time.Now())
	if err := i.triggerRepo.Save(ctx, trigger); err != nil {
		log.Errorf("Failed to update last triggered time for trigger %s: %v", trigger.ID(), err)
		// Don't fail the job creation for this @pyshx
	}

	if err := i.job.StartMonitoring(ctx, j, nil); err != nil {
		log.Errorf("Failed to start monitoring for time-driven job %s: %v", j.ID(), err)
		return nil, err
	}

	tx.Commit()
	return j, nil
}

func (i *Trigger) Update(ctx context.Context, param interfaces.UpdateTriggerParam) (_ *trigger.Trigger, err error) {
	if err := i.checkPermission(ctx, rbac.ActionEdit); err != nil {
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

	t, err := i.triggerRepo.FindByID(ctx, param.ID)
	if err != nil {
		return nil, err
	}

	originalEventSource := t.EventSource()

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

	if i.scheduler != nil {
		if originalEventSource == "TIME_DRIVEN" && t.EventSource() != "TIME_DRIVEN" {
			if err := i.scheduler.DeleteScheduledJob(ctx, t.ID()); err != nil {
				log.Errorf("Failed to delete scheduled job for trigger %s: %v", t.ID(), err)
			}
		} else if originalEventSource != "TIME_DRIVEN" && t.EventSource() == "TIME_DRIVEN" {
			if err := i.scheduler.CreateScheduledJob(ctx, t); err != nil {
				log.Errorf("Failed to create scheduled job for trigger %s: %v", t.ID(), err)
			}
		} else if originalEventSource == "TIME_DRIVEN" && t.EventSource() == "TIME_DRIVEN" {
			if err := i.scheduler.UpdateScheduledJob(ctx, t); err != nil {
				log.Errorf("Failed to update scheduled job for trigger %s: %v", t.ID(), err)
			}
		}
	}

	tx.Commit()
	return t, nil
}

func (i *Trigger) Delete(ctx context.Context, id id.TriggerID) (err error) {
	if err := i.checkPermission(ctx, rbac.ActionDelete); err != nil {
		return err
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

	trigger, err := i.triggerRepo.FindByID(ctx, id)
	if err != nil {
		return err
	}

	if trigger.EventSource() == "TIME_DRIVEN" && i.scheduler != nil {
		if err := i.scheduler.DeleteScheduledJob(ctx, id); err != nil {
			log.Errorf("Failed to delete scheduled job for trigger %s: %v", id, err)
		}
	}

	if err := i.triggerRepo.Remove(ctx, id); err != nil {
		return err
	}

	tx.Commit()
	return nil
}
