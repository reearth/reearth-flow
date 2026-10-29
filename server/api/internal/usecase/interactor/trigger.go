package interactor

import (
	"context"
	"errors"
	"fmt"
	"maps"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
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
	transaction       usecasex.Transactor
	batch             gateway.Batch
	file              gateway.File
	job               interfaces.Job
	scheduler         gateway.Scheduler
	permissionChecker gateway.PermissionChecker
	planLimits        planLimits
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
		planLimits:        newPlanLimits(r, gr, jobUsecase),
	}
}

func (i *Trigger) checkPermission(ctx context.Context, action string, workspaceID ...accountsid.WorkspaceID) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceTrigger, action, workspaceID...)
}

func (i *Trigger) Fetch(ctx context.Context, ids []id.TriggerID) ([]*trigger.Trigger, error) {
	got, err := i.triggerRepo.FindByIDs(ctx, ids)
	if err != nil {
		return nil, err
	}
	triggers := alignToRequestedIDs(ids, got, nil, (*trigger.Trigger).ID)

	if err := authorizeFetchByWorkspace(ctx, i.checkPermission, rbac.ActionAny, triggers, nil, (*trigger.Trigger).Workspace); err != nil {
		return nil, err
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
	if param.EventSource == "TIME_DRIVEN" && param.TimeInterval == trigger.TimeIntervalEveryHour {
		if err := i.planLimits.hourlyAllowed(ctx, param.WorkspaceID); err != nil {
			return nil, err
		}
	}

	var trg *trigger.Trigger
	if err := i.transaction.WithinTransaction(ctx, func(ctx context.Context) error {
		d, err := i.ensureDeploymentInWorkspace(ctx, param.DeploymentID, param.WorkspaceID)
		if err != nil {
			return err
		}
		// An API call can fill a required variable itself, so only its runs
		// check them.
		if param.EventSource != "API_DRIVEN" {
			if err := ensureTriggerRequiredVariables(d, param.Variables); err != nil {
				return err
			}
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

		switch param.EventSource {
		case "TIME_DRIVEN":
			t = t.TimeInterval(trigger.TimeInterval(param.TimeInterval))
		case "API_DRIVEN":
			t = t.AuthToken(param.AuthToken)
		}

		if len(param.Variables) > 0 {
			t = t.Variables(param.Variables)
		}

		trg, err = t.Build()
		if err != nil {
			return err
		}

		return i.triggerRepo.Save(ctx, trg)
	}); err != nil {
		return nil, err
	}

	if trg.EventSource() == "TIME_DRIVEN" && i.scheduler != nil {
		if err := i.scheduler.CreateScheduledJob(ctx, trg); err != nil {
			log.Errorf("Failed to create scheduled job for trigger %s: %v", trg.ID(), err)
			// Don't fail trigger creation if scheduler registration fails
			// The trigger can still be executed manually @pyshx
		}
	}

	return trg, nil
}

func (i *Trigger) ExecuteAPITrigger(ctx context.Context, p interfaces.ExecuteAPITriggerParam) (_ *job.Job, err error) {
	trigger, err := i.triggerRepo.FindByID(ctx, p.TriggerID)
	if err != nil {
		return nil, err
	}
	if trigger == nil {
		return nil, rerror.ErrNotFound
	}

	if !trigger.Enabled() {
		return nil, fmt.Errorf("trigger is disabled")
	}

	// API-driven triggers use their own secret token as the auth mechanism.
	// No workspace membership is required — the token is sufficient proof of authorization.
	if trigger.EventSource() == "API_DRIVEN" {
		if trigger.AuthToken() == nil || p.AuthenticationToken != *trigger.AuthToken() {
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
	if deployment == nil {
		return nil, rerror.ErrNotFound
	}

	// The deployment's own copy of the variables, not the project's current
	// ones, so a run matches the workflow file that was deployed.
	projectParams := projectParametersToMap(parameter.NewParameterList(deployment.Parameters()))

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
	if err := ensureRequiredParameters(deployment.Parameters(), finalVarMap); err != nil {
		return nil, err
	}

	// The job is saved under the deployment's workspace, so that is the one
	// whose limit applies, even for a trigger saved before cross-workspace
	// references were refused. Only calls authenticated by the trigger's token
	// count toward the daily API trigger calls, and the variables are checked
	// first so a call they refuse is not counted.
	var admission jobAdmission
	if trigger.EventSource() == "API_DRIVEN" {
		admission, err = i.planLimits.ensureCanRunAPITrigger(ctx, deployment.Workspace())
	} else {
		admission, err = i.planLimits.ensureCanRun(ctx, deployment.Workspace())
	}
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

	// From here a failure is the server's, so the call is given back. A call
	// refused because the workspace's jobs are running counts, like any refused
	// call.
	defer func() {
		if err != nil && !errors.Is(err, interfaces.ErrJobAlreadyRunning) {
			admission.refundCall(ctx)
		}
	}()

	metadataURL, err := i.file.UploadMetadata(ctx, j.ID().String(), []string{})
	if err != nil {
		return nil, err
	}
	if metadataURL != nil {
		j.SetMetadataURL(metadataURL.String())
	}

	// Update last triggered time
	trigger.SetLastTriggered(time.Now())

	if err := i.saveTriggeredJob(ctx, admission, j, trigger); err != nil {
		return nil, err
	}

	var projectID id.ProjectID
	if deployment.Project() != nil {
		projectID = *deployment.Project()
	}

	// Submitted after the save, so a failure here leaves a job the user can see
	// rather than an untracked cloud job.
	gcpJobID, err := i.batch.SubmitJob(ctx, j.ID(), deployment.WorkflowURL(), j.MetadataURL(), variable.ToWorkerMap(finalVarMap), projectID, deployment.Workspace(), nil, nil)
	if err != nil {
		log.Debugfc(ctx, "[Trigger] Job submission failed: %v\n", err)
		failJob(ctx, i.jobRepo, j)
		return nil, interfaces.ErrJobCreationFailed
	}

	j.SetGCPJobID(gcpJobID)
	if err := i.jobRepo.Save(ctx, j); err != nil {
		log.Errorf("Failed to save job %s with GCP ID: %v", j.ID(), err)
		return nil, err
	}
	// A trigger run with its token is an API call; any other is a user running
	// it by hand.
	jobTrigger := jobTriggerManual
	if trigger.EventSource() == "API_DRIVEN" {
		jobTrigger = jobTriggerAPI
	}
	logJobEvent(ctx, jobEventSubmitted, j, "trigger", jobTrigger, "trigger_id", trigger.ID().String())

	if err := i.job.StartMonitoring(ctx, j, p.NotificationURL); err != nil {
		log.Errorf("Failed to start monitoring for job %s: %v", j.ID(), err)
		return nil, err
	}

	return j, nil
}

func (i *Trigger) ExecuteTimeDrivenTrigger(ctx context.Context, p interfaces.ExecuteTimeDrivenTriggerParam) (_ *job.Job, err error) {
	trigger, err := i.triggerRepo.FindByID(ctx, p.TriggerID)
	if err != nil {
		return nil, err
	}
	if trigger == nil {
		return nil, rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionCreate, trigger.Workspace()); err != nil {
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
	if deployment == nil {
		return nil, rerror.ErrNotFound
	}

	// The deployment's own copy of the variables, not the project's current
	// ones, so a run matches the workflow file that was deployed.
	projectParams := projectParametersToMap(parameter.NewParameterList(deployment.Parameters()))

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
	if err := ensureRequiredParameters(deployment.Parameters(), finalVarMap); err != nil {
		return nil, err
	}

	// The job is saved under the deployment's workspace, so that is the one
	// whose limit applies, even for a trigger saved before cross-workspace
	// references were refused.
	admission, err := i.planLimits.ensureCanRun(ctx, deployment.Workspace())
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
	if metadataURL != nil {
		j.SetMetadataURL(metadataURL.String())
	}

	// Update last triggered time
	trigger.SetLastTriggered(time.Now())

	if err := i.saveTriggeredJob(ctx, admission, j, trigger); err != nil {
		return nil, err
	}

	var projectID id.ProjectID
	if deployment.Project() != nil {
		projectID = *deployment.Project()
	}

	// Submitted after the save, so a failure here leaves a job the user can see
	// rather than an untracked cloud job.
	gcpJobID, err := i.batch.SubmitJob(ctx, j.ID(), deployment.WorkflowURL(), j.MetadataURL(), variable.ToWorkerMap(finalVarMap), projectID, deployment.Workspace(), nil, nil)
	if err != nil {
		log.Debugfc(ctx, "[Trigger] Time-driven job submission failed: %v\n", err)
		failJob(ctx, i.jobRepo, j)
		return nil, interfaces.ErrJobCreationFailed
	}

	j.SetGCPJobID(gcpJobID)
	if err := i.jobRepo.Save(ctx, j); err != nil {
		log.Errorf("Failed to save time-driven job %s with GCP ID: %v", j.ID(), err)
		return nil, err
	}
	logJobEvent(ctx, jobEventSubmitted, j, "trigger", jobTriggerSchedule, "trigger_id", trigger.ID().String())

	if err := i.job.StartMonitoring(ctx, j, nil); err != nil {
		log.Errorf("Failed to start monitoring for time-driven job %s: %v", j.ID(), err)
		return nil, err
	}

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
	// The plan is asked before the transaction so the call never holds one open,
	// but whether the update makes the trigger hourly is decided from the
	// trigger as read inside it. A trigger that is hourly already needs no
	// answer.
	wantsHourly := param.EventSource == "TIME_DRIVEN" && param.TimeInterval == trigger.TimeIntervalEveryHour
	var hourlyErr error
	if wantsHourly && !isHourly(trg) {
		hourlyErr = i.planLimits.hourlyAllowed(ctx, trg.Workspace())
	}

	var t *trigger.Trigger
	if err := i.transaction.WithinTransaction(ctx, func(ctx context.Context) error {
		var err error
		t, err = i.triggerRepo.FindByID(ctx, param.ID)
		if err != nil {
			return err
		}

		// Only a change is refused, so a trigger made hourly before a downgrade
		// keeps its schedule; it just cannot be made hourly again.
		if wantsHourly && !isHourly(t) {
			if hourlyErr == nil && isHourly(trg) {
				// It stopped being hourly after the plan was skipped.
				hourlyErr = i.planLimits.hourlyAllowed(ctx, t.Workspace())
			}
			if hourlyErr != nil {
				return hourlyErr
			}
		}

		originalEventSource := t.EventSource()

		// Checked before anything changes, so a refused update leaves the
		// trigger as it was.
		var d *deployment.Deployment
		if param.DeploymentID != nil {
			d, err = i.ensureDeploymentInWorkspace(ctx, *param.DeploymentID, t.Workspace())
		} else {
			d, err = i.deploymentRepo.FindByID(ctx, t.Deployment())
			if err == nil && d == nil {
				err = rerror.ErrNotFound
			}
		}
		if err != nil {
			return err
		}
		vars := t.Variables()
		if param.Variables != nil {
			vars = param.Variables
		}
		source := t.EventSource()
		if param.EventSource == "TIME_DRIVEN" || param.EventSource == "API_DRIVEN" {
			source = param.EventSource
		}
		if source != "API_DRIVEN" {
			if err := ensureTriggerRequiredVariables(d, vars); err != nil {
				return err
			}
		}

		if param.DeploymentID != nil {
			t.SetDeployment(*param.DeploymentID)
		}

		if param.Description != nil {
			t.SetDescription(*param.Description)
		}

		switch param.EventSource {
		case "TIME_DRIVEN":
			t.SetEventSource(trigger.EventSourceType(param.EventSource))
			t.SetTimeInterval(trigger.TimeInterval(param.TimeInterval))
			t.SetAuthToken("")
		case "API_DRIVEN":
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
			return err
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

		return nil
	}); err != nil {
		return nil, err
	}
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

	return i.transaction.WithinTransaction(ctx, func(ctx context.Context) error {
		trigger, err := i.triggerRepo.FindByID(ctx, id)
		if err != nil {
			return err
		}

		if trigger.EventSource() == "TIME_DRIVEN" && i.scheduler != nil {
			if err := i.scheduler.DeleteScheduledJob(ctx, id); err != nil {
				log.Errorf("Failed to delete scheduled job for trigger %s: %v", id, err)
			}
		}

		return i.triggerRepo.Remove(ctx, id)
	})
}

// saveTriggeredJob saves a trigger's new job and then the trigger's last
// triggered time. They are not in a transaction: the job save reserves the
// plan's slot itself, and a transaction would make it refuse runs that start
// together, such as daily triggers at midnight. A failure to save the trigger
// does not fail the run.
func (i *Trigger) saveTriggeredJob(ctx context.Context, admission jobAdmission, j *job.Job, t *trigger.Trigger) error {
	if err := admission.save(ctx, j); err != nil {
		return err
	}
	if err := i.triggerRepo.Save(ctx, t); err != nil {
		log.Errorf("Failed to update last triggered time for trigger %s: %v", t.ID(), err)
	}
	return nil
}

// ensureDeploymentInWorkspace refuses a deployment that is missing or belongs
// to another workspace than the trigger.
func (i *Trigger) ensureDeploymentInWorkspace(ctx context.Context, did id.DeploymentID, wid accountsid.WorkspaceID) (*deployment.Deployment, error) {
	d, err := i.deploymentRepo.FindByID(ctx, did)
	if err != nil {
		return nil, err
	}
	if d == nil {
		return nil, rerror.ErrNotFound
	}
	if d.Workspace() != wid {
		return nil, interfaces.ErrDeploymentNotInWorkspace
	}
	return d, nil
}

// ensureTriggerRequiredVariables refuses a trigger whose runs would leave a
// required variable empty: neither the deployment's default nor the trigger's
// own value fills it.
func ensureTriggerRequiredVariables(d *deployment.Deployment, vars []variable.Variable) error {
	params := d.Parameters()
	resolved, err := resolveVariables(
		ModeTimeDriven,
		projectParametersToMap(parameter.NewParameterList(params)),
		variable.SliceToMap(vars),
		nil,
	)
	if err != nil {
		return err
	}
	return ensureRequiredParameters(params, resolved)
}

func isHourly(t *trigger.Trigger) bool {
	if t.EventSource() != "TIME_DRIVEN" {
		return false
	}
	ti := t.TimeInterval()
	return ti != nil && *ti == trigger.TimeIntervalEveryHour
}
