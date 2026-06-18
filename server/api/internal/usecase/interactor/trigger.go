package interactor

import (
	"context"
	"fmt"
	"maps"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearth-flow/api/pkg/variable"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
)

type Trigger struct {
	triggerRepo       repo.Trigger
	deploymentRepo    repo.Deployment
	jobRepo           repo.Job
	workerConfigRepo  repo.WorkerConfig
	paramRepo         repo.Parameter
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
		paramRepo:         r.Parameter,
		transaction:       r.Transaction,
		batch:             gr.Batch,
		file:              gr.File,
		job:               jobUsecase,
		scheduler:         gr.Scheduler,
		permissionChecker: permissionChecker,
	}
}

func (i *Trigger) checkPermission(ctx context.Context, action string, workspaceID ...accountsid.WorkspaceID) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceTrigger, action, workspaceID...)
}

func (i *Trigger) Fetch(ctx context.Context, ids []id.TriggerID) ([]*trigger.Trigger, error) {
	triggers, err := i.triggerRepo.FindByIDs(ctx, ids)
	if err != nil {
		return nil, err
	}

	if len(triggers) == 0 {
		if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
			return nil, err
		}
	} else {
		if err := i.checkPermission(ctx, rbac.ActionAny, triggers[0].Workspace()); err != nil { // single-workspace batch assumption
			return nil, err
		}
	}

	return triggers, nil
}

func (i *Trigger) FindByWorkspace(ctx context.Context, id accountsid.WorkspaceID, p *interfaces.PaginationParam, keyword *string) ([]*trigger.Trigger, *interfaces.PageBasedInfo, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny, id); err != nil {
		return nil, nil, err
	}

	return i.triggerRepo.FindByWorkspace(ctx, id, p, keyword)
}

func (i *Trigger) FindByID(ctx context.Context, id id.TriggerID) (*trigger.Trigger, error) {
	t, err := i.triggerRepo.FindByID(ctx, id)
	if err != nil {
		return nil, err
	}
	if t == nil {
		return nil, rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionAny, t.Workspace()); err != nil {
		return nil, err
	}

	return t, nil
}

func (i *Trigger) Create(ctx context.Context, param interfaces.CreateTriggerParam) (result *trigger.Trigger, err error) {
	if err := i.checkPermission(ctx, rbac.ActionCreate, param.WorkspaceID); err != nil {
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
		Enabled(param.Enabled).
		CreatedAt(time.Now()).
		UpdatedAt(time.Now())

	if param.EventSource == "TIME_DRIVEN" {
		t = t.TimeInterval(trigger.TimeInterval(param.TimeInterval))
	} else if param.EventSource == "API_DRIVEN" {
		t = t.AuthToken(param.AuthToken)
	}

	if len(param.Variables) > 0 {
		t = t.Variables(param.Variables)
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

	if !trigger.Enabled() {
		return nil, fmt.Errorf("trigger is disabled")
	}

	// API-driven triggers use their own secret token as the auth mechanism.
	// No workspace membership is required — the token is sufficient proof of authorization.
	if trigger.EventSource() == "API_DRIVEN" {
		if p.AuthenticationToken != *trigger.AuthToken() {
			return nil, fmt.Errorf("invalid auth token")
		}
	} else {
		if err := i.checkPermission(ctx, rbac.ActionCreate, trigger.Workspace()); err != nil {
			return nil, err
		}
	}

	deployment, err := i.deploymentRepo.FindByID(ctx, trigger.Deployment())
	if err != nil {
		return nil, err
	}

	var projectParams map[string]variable.Variable
	if deployment.Project() != nil {
		pls, err := i.paramRepo.FindByProject(ctx, *deployment.Project())
		if err != nil {
			return nil, err
		}
		projectParams = projectParametersToMap(pls)
	}

	var triggerVars map[string]variable.Variable
	if tvs := trigger.Variables(); len(tvs) > 0 {
		triggerVars = variable.SliceToMap(tvs)
	}

	schema := map[string]variable.Variable{}
	maps.Copy(schema, projectParams)
	maps.Copy(schema, triggerVars)

	requestVars := normalizeRequestVars(p.Variables, schema)

	finalVarMap, err := resolveVariables(
		ModeAPIDriven,
		projectParams,
		triggerVars,
		requestVars,
	)
	if err != nil {
		return nil, err
	}

	deploymentID1 := deployment.ID()

	j, err := job.New().
		NewID().
		Deployment(&deploymentID1).
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

	gcpJobID, err := i.batch.SubmitJob(ctx, j.ID(), deployment.WorkflowURL(), j.MetadataURL(), variable.ToWorkerMap(finalVarMap), projectID, deployment.Workspace(), nil, nil)
	if err != nil {
		log.Debugfc(ctx, "[Trigger] Job submission failed: %v\n", err)
		return nil, interfaces.ErrJobCreationFailed
	}

	j.SetGCPJobID(gcpJobID)
	if err := i.jobRepo.Save(ctx, j); err != nil {
		log.Errorf("Failed to save job %s with GCP ID: %v", j.ID(), err)
		return nil, err
	}

	// Update last triggered time
	trigger.SetLastTriggered(time.Now())
	if err := i.triggerRepo.Save(ctx, trigger); err != nil {
		log.Errorf("Failed to update last triggered time for trigger %s: %v", trigger.ID(), err)
		// Don't fail the job creation for this @pyshx
	}

	if err := i.job.StartMonitoring(ctx, j, p.NotificationURL); err != nil {
		log.Errorf("Failed to start monitoring for job %s: %v", j.ID(), err)
		return nil, err
	}

	tx.Commit()
	return j, nil
}

func (i *Trigger) ExecuteTimeDrivenTrigger(ctx context.Context, p interfaces.ExecuteTimeDrivenTriggerParam) (_ *job.Job, err error) {
	trg, err := i.triggerRepo.FindByID(ctx, p.TriggerID)
	if err != nil {
		return nil, err
	}
	if trg == nil {
		return nil, rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionCreate, trg.Workspace()); err != nil {
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

	if !trigger.Enabled() {
		return nil, fmt.Errorf("trigger is disabled")
	}

	if trigger.EventSource() != "TIME_DRIVEN" {
		return nil, fmt.Errorf("trigger is not time-driven")
	}

	deployment, err := i.deploymentRepo.FindByID(ctx, trigger.Deployment())
	if err != nil {
		return nil, err
	}

	var projectParams map[string]variable.Variable
	if deployment.Project() != nil {
		pls, err := i.paramRepo.FindByProject(ctx, *deployment.Project())
		if err != nil {
			return nil, err
		}
		projectParams = projectParametersToMap(pls)
	}

	var triggerVars map[string]variable.Variable
	if tvs := trigger.Variables(); len(tvs) > 0 {
		triggerVars = variable.SliceToMap(tvs)
	}

	finalVarMap, err := resolveVariables(
		ModeTimeDriven,
		projectParams,
		triggerVars,
		nil,
	)
	if err != nil {
		return nil, err
	}

	deploymentID2 := deployment.ID()

	j, err := job.New().
		NewID().
		Deployment(&deploymentID2).
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

	gcpJobID, err := i.batch.SubmitJob(ctx, j.ID(), deployment.WorkflowURL(), j.MetadataURL(), variable.ToWorkerMap(finalVarMap), projectID, deployment.Workspace(), nil, nil)
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
	trg, err := i.triggerRepo.FindByID(ctx, param.ID)
	if err != nil {
		return nil, err
	}
	if trg == nil {
		return nil, rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionEdit, trg.Workspace()); err != nil {
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

	if param.Enabled != nil {
		t.SetEnabled(*param.Enabled)
	}

	if param.Variables != nil {
		t.SetVariables(param.Variables)
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
	trg, err := i.triggerRepo.FindByID(ctx, id)
	if err != nil {
		return err
	}
	if trg == nil {
		return rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionDelete, trg.Workspace()); err != nil {
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
