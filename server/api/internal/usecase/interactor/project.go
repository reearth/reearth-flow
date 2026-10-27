package interactor

import (
	"context"
	"errors"
	"fmt"
	"time"

	gqlworkspace "github.com/reearth/reearth-accounts/server/pkg/gqlclient/workspace"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
)

type Project struct {
	assetRepo         repo.Asset
	workflowRepo      repo.Workflow
	projectRepo       repo.Project
	jobRepo           repo.Job
	workerConfigRepo  repo.WorkerConfig
	workspaceRepo     gqlworkspace.WorkspaceRepo
	transaction       usecasex.Transactor
	file              gateway.File
	batch             gateway.Batch
	cloudRunWorker    gateway.CloudRunWorker
	websocket         interfaces.WebsocketClient
	job               interfaces.Job
	permissionChecker gateway.PermissionChecker
	policyChecker     gateway.PolicyChecker
	planLimits        planLimits
}

func NewProject(r *repo.Container, gr *gateway.Container, jobUsecase interfaces.Job, permissionChecker gateway.PermissionChecker, workspaceRepo gqlworkspace.WorkspaceRepo, websocket interfaces.WebsocketClient) interfaces.Project {
	return &Project{
		assetRepo:         r.Asset,
		workflowRepo:      r.Workflow,
		projectRepo:       r.Project,
		jobRepo:           r.Job,
		workerConfigRepo:  r.WorkerConfig,
		workspaceRepo:     workspaceRepo,
		transaction:       r.Transaction,
		file:              gr.File,
		batch:             gr.Batch,
		cloudRunWorker:    gr.CloudRunWorker,
		websocket:         websocket,
		job:               jobUsecase,
		permissionChecker: permissionChecker,
		policyChecker:     gr.PolicyChecker,
		planLimits:        newPlanLimits(r, gr, jobUsecase),
	}
}

func (i *Project) checkPermission(ctx context.Context, action string, workspaceID ...accountsid.WorkspaceID) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceProject, action, workspaceID...)
}

func (i *Project) Fetch(ctx context.Context, ids []id.ProjectID) ([]*project.Project, error) {
	got, err := i.projectRepo.FindByIDs(ctx, ids)
	if err != nil {
		return nil, err
	}
	projects := alignToRequestedIDs(ids, got, nil, (*project.Project).ID)

	if err := authorizeFetchByWorkspace(ctx, i.checkPermission, rbac.ActionList, projects, nil, (*project.Project).Workspace); err != nil {
		return nil, err
	}

	return projects, nil
}

func (i *Project) FindByWorkspace(ctx context.Context, id accountsid.WorkspaceID, pagination *interfaces.PaginationParam, keyword *string, includeArchived *bool, createdBy accountsid.UserIDList) ([]*project.Project, *interfaces.PageBasedInfo, error) {
	if err := i.checkPermission(ctx, rbac.ActionList, id); err != nil {
		return nil, nil, err
	}

	return i.projectRepo.FindByWorkspace(ctx, id, pagination, keyword, includeArchived, createdBy)
}

func (i *Project) FindViewable(ctx context.Context, pid id.ProjectID) (*project.Project, error) {
	p, err := i.projectRepo.FindByID(ctx, pid)
	if err != nil {
		return nil, err
	}
	return i.viewable(ctx, p)
}

func (i *Project) FindViewableByAlias(ctx context.Context, wid accountsid.WorkspaceID, alias string) (*project.Project, error) {
	p, err := i.projectRepo.FindByAlias(ctx, wid, alias)
	if err != nil {
		return nil, err
	}
	return i.viewable(ctx, p)
}

func (i *Project) Search(ctx context.Context, p interfaces.ProjectSearchParam) ([]*project.Project, *interfaces.PageBasedInfo, error) {
	p.Page, p.PageSize = interfaces.NormalizePage(p.Page, p.PageSize)

	if p.Workspace != nil {
		member, err := i.isMember(ctx, *p.Workspace)
		if err != nil {
			return nil, nil, err
		}
		if member {
			return i.projectRepo.Search(ctx, p)
		}
	}

	public := project.VisibilityPublic
	if p.Visibility != nil && *p.Visibility != public {
		return nil, interfaces.NewPageBasedInfo(0, p.Page, p.PageSize), nil
	}
	p.Visibility = &public
	p.IncludeArchived = false
	return i.projectRepo.Search(ctx, p)
}

// viewable returns p if the caller may see it, and rerror.ErrNotFound if not.
func (i *Project) viewable(ctx context.Context, p *project.Project) (*project.Project, error) {
	if p == nil {
		return nil, rerror.ErrNotFound
	}
	if isOpen(p) {
		return p, nil
	}
	member, err := i.isMember(ctx, p.Workspace())
	if err != nil {
		return nil, err
	}
	if !member {
		return nil, rerror.ErrNotFound
	}
	return p, nil
}

// isMember reports whether the caller may list the workspace's projects, which
// every workspace role may. A caller with no user is never a member. Only a
// denial means no; other errors are returned.
func (i *Project) isMember(ctx context.Context, wid accountsid.WorkspaceID) (bool, error) {
	if adapter.User(ctx) == nil {
		return false, nil
	}
	err := i.checkPermission(ctx, rbac.ActionList, wid)
	if errors.Is(err, interfaces.ErrOperationDenied) {
		return false, nil
	}
	return err == nil, err
}

// isOpen reports whether anyone may see p: it is public and not archived.
func isOpen(p *project.Project) bool {
	return p.Visibility() == project.VisibilityPublic && !p.IsArchived()
}

func (i *Project) Create(ctx context.Context, p interfaces.CreateProjectParam) (_ *project.Project, err error) {
	if err := i.checkPermission(ctx, rbac.ActionCreate, p.WorkspaceID); err != nil {
		return nil, err
	}
	if err := i.ensureVisibilityAllowed(ctx, p.WorkspaceID, p.Visibility); err != nil {
		return nil, err
	}

	var proj *project.Project
	if err := i.transaction.WithinTransaction(ctx, func(ctx context.Context) error {
		if _, err := i.workspaceRepo.FindByID(ctx, p.WorkspaceID.String()); err != nil {
			return err
		}

		// New projects are public unless the caller says otherwise.
		uid := callerID(ctx)
		pb := project.New().
			NewID().
			Workspace(p.WorkspaceID).
			Visibility(project.VisibilityPublic).
			CreatedBy(uid).
			UpdatedBy(uid)
		if p.Name != nil {
			pb = pb.Name(*p.Name)
		}
		if p.Description != nil {
			pb = pb.Description(*p.Description)
		}
		if p.Archived != nil {
			pb = pb.IsArchived(*p.Archived)
		}

		var err error
		proj, err = pb.Build()
		if err != nil {
			return err
		}

		if err := applyProjectMetadata(proj, projectMetadataParam{
			Alias:      p.Alias,
			Visibility: p.Visibility,
			Readme:     p.Readme,
			License:    p.License,
			Topics:     p.Topics,
		}); err != nil {
			return err
		}
		if p.Alias != nil {
			if err := i.ensureAliasAvailable(ctx, proj); err != nil {
				return err
			}
		}

		return i.projectRepo.Save(ctx, proj)
	}); err != nil {
		return nil, err
	}
	return proj, nil
}

func (i *Project) Update(ctx context.Context, p interfaces.UpdateProjectParam) (_ *project.Project, err error) {
	proj, err := i.projectRepo.FindByID(ctx, p.ID)
	if err != nil {
		return nil, err
	}
	if proj == nil {
		return nil, rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionEdit, proj.Workspace()); err != nil {
		return nil, err
	}
	// The plan is asked before the transaction so the call never holds one open,
	// but the decision below uses the project as read inside it: re-sending
	// PRIVATE must not pass because the project was private when first read and
	// has been made public since.
	wantsPrivate := p.Visibility != nil && *p.Visibility == project.VisibilityPrivate
	var privateAllowed bool
	var privateErr error
	if wantsPrivate {
		privateAllowed, privateErr = i.canCreatePrivate(ctx, proj.Workspace())
	}

	var prj *project.Project
	if err := i.transaction.WithinTransaction(ctx, func(ctx context.Context) error {
		var err error
		prj, err = i.projectRepo.FindByID(ctx, p.ID)
		if err != nil {
			return err
		}

		// Only a change is refused, so a project made private on a paid plan
		// stays private after a downgrade; it just cannot be made private again.
		if wantsPrivate && prj.Visibility() != project.VisibilityPrivate {
			if privateErr != nil {
				return privateErr
			}
			if !privateAllowed {
				return interfaces.ErrPrivateProjectNotAllowed
			}
		}

		editedAt := prj.UpdatedAt()
		if p.Name != nil {
			prj.SetUpdateName(*p.Name)
		}
		if p.Description != nil {
			prj.SetUpdateDescription(*p.Description)
		}
		if p.Archived != nil {
			prj.SetArchived(*p.Archived)
		}
		if p.IsBasicAuthActive != nil {
			prj.SetIsBasicAuthActive(*p.IsBasicAuthActive)
		}
		if p.IsLocked != nil {
			prj.SetIsLocked(*p.IsLocked)
		}
		if p.BasicAuthUsername != nil {
			prj.SetBasicAuthUsername(*p.BasicAuthUsername)
		}
		if p.BasicAuthPassword != nil {
			prj.SetBasicAuthPassword(*p.BasicAuthPassword)
		}
		if err := applyProjectMetadata(prj, projectMetadataParam{
			Alias:      p.Alias,
			Visibility: p.Visibility,
			Readme:     p.Readme,
			License:    p.License,
			Topics:     p.Topics,
		}); err != nil {
			return err
		}
		if p.Alias != nil {
			if err := i.ensureAliasAvailable(ctx, prj); err != nil {
				return err
			}
		}
		// Every setter bumps updatedAt, so an unchanged time means the input set
		// nothing and the last editor stays as it was.
		if !prj.UpdatedAt().Equal(editedAt) {
			stampUpdatedBy(ctx, prj)
		}

		return i.projectRepo.Save(ctx, prj)
	}); err != nil {
		return nil, err
	}
	return prj, nil
}

// callerID is the signed-in user's ID, or nil when the call has no user.
func callerID(ctx context.Context) *accountsid.UserID {
	u := adapter.User(ctx)
	if u == nil {
		return nil
	}
	uid := u.ID()
	return &uid
}

// stampUpdatedBy records the caller as p's last editor. A call with no user
// leaves the previous editor in place rather than clearing it.
func stampUpdatedBy(ctx context.Context, p *project.Project) {
	if uid := callerID(ctx); uid != nil {
		p.SetUpdatedBy(uid)
	}
}

func (i *Project) CanCreatePrivate(ctx context.Context, wid accountsid.WorkspaceID) (bool, error) {
	if err := i.checkPermission(ctx, rbac.ActionRead, wid); err != nil {
		return false, err
	}
	return i.canCreatePrivate(ctx, wid)
}

// CanScheduleHourly gives the same answer Trigger.Create and Trigger.Update act
// on, so the UI offers an hourly schedule exactly when saving one would pass.
func (i *Project) CanScheduleHourly(ctx context.Context, wid accountsid.WorkspaceID) (bool, error) {
	if err := i.checkPermission(ctx, rbac.ActionRead, wid); err != nil {
		return false, err
	}
	return i.planLimits.hourlyAllowed(ctx, wid) == nil, nil
}

// canCreatePrivate asks the plan policy. Without a policy checker every
// workspace may have private projects.
func (i *Project) canCreatePrivate(ctx context.Context, wid accountsid.WorkspaceID) (bool, error) {
	if i.policyChecker == nil {
		return true, nil
	}
	res, err := i.policyChecker.CheckPolicy(ctx, gateway.PolicyCheckRequest{
		CheckType:   gateway.PolicyCheckPrivateProjectCreation,
		WorkspaceID: wid,
		Value:       1,
	})
	if err != nil {
		return false, err
	}
	return res.Allowed, nil
}

// ensureVisibilityAllowed rejects making a project private when the plan does
// not allow it. Public is always allowed. It runs before any transaction so the
// policy call never holds one open.
func (i *Project) ensureVisibilityAllowed(ctx context.Context, wid accountsid.WorkspaceID, v *project.Visibility) error {
	if v == nil || *v != project.VisibilityPrivate {
		return nil
	}
	allowed, err := i.canCreatePrivate(ctx, wid)
	if err != nil {
		return err
	}
	if !allowed {
		return interfaces.ErrPrivateProjectNotAllowed
	}
	return nil
}

func (i *Project) CheckAlias(ctx context.Context, wid accountsid.WorkspaceID, alias string, pid *id.ProjectID) (bool, error) {
	if err := i.checkPermission(ctx, rbac.ActionRead, wid); err != nil {
		return false, err
	}

	var own id.ProjectID
	if pid != nil {
		own = *pid
	}
	if err := project.ValidateAlias(alias, own); err != nil {
		return false, err
	}

	other, err := i.projectRepo.FindByAlias(ctx, wid, alias)
	if errors.Is(err, rerror.ErrNotFound) {
		return true, nil
	}
	if err != nil {
		return false, err
	}
	return pid != nil && other.ID() == *pid, nil
}

func (i *Project) SetStar(ctx context.Context, pid id.ProjectID, starred bool) (*project.Project, error) {
	u := adapter.User(ctx)
	if u == nil {
		return nil, interfaces.ErrOperationDenied
	}

	proj, err := i.projectRepo.FindByID(ctx, pid)
	if err != nil {
		return nil, err
	}
	if !isOpen(proj) {
		if err := i.checkPermission(ctx, rbac.ActionRead, proj.Workspace()); err != nil {
			return nil, err
		}
	}

	if err := i.projectRepo.SetStar(ctx, pid, accountsid.UserID(u.ID()), starred); err != nil {
		return nil, err
	}
	return i.projectRepo.FindByID(ctx, pid)
}

// ensureAliasAvailable gives a clear error before the write. A race between two
// writers is decided by the unique (workspace, alias) index in either database.
func (i *Project) ensureAliasAvailable(ctx context.Context, p *project.Project) error {
	other, err := i.projectRepo.FindByAlias(ctx, p.Workspace(), p.Alias())
	if errors.Is(err, rerror.ErrNotFound) {
		return nil
	}
	if err != nil {
		return err
	}
	if other.ID() != p.ID() {
		return interfaces.ErrProjectAliasAlreadyUsed
	}
	return nil
}

type projectMetadataParam struct {
	Alias      *string
	Visibility *project.Visibility
	Readme     *string
	License    *string
	Topics     []string
}

func applyProjectMetadata(p *project.Project, m projectMetadataParam) error {
	if m.Alias != nil {
		if err := p.SetAlias(*m.Alias); err != nil {
			return err
		}
	}
	if m.Visibility != nil {
		if err := p.SetVisibility(*m.Visibility); err != nil {
			return err
		}
	}
	if m.Readme != nil {
		p.SetReadme(*m.Readme)
	}
	if m.License != nil {
		p.SetLicense(*m.License)
	}
	if m.Topics != nil {
		if err := p.SetTopics(m.Topics); err != nil {
			return err
		}
	}
	return nil
}

func (i *Project) Delete(ctx context.Context, projectID id.ProjectID) (err error) {
	proj, err := i.projectRepo.FindByID(ctx, projectID)
	if err != nil {
		return err
	}
	if proj == nil {
		return rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionDelete, proj.Workspace()); err != nil {
		return err
	}

	if err := i.transaction.WithinTransaction(ctx, func(ctx context.Context) error {
		prj, err := i.projectRepo.FindByID(ctx, projectID)
		if err != nil {
			return err
		}

		deleter := ProjectDeleter{
			File:    i.file,
			Project: i.projectRepo,
		}
		if err := deleter.Delete(ctx, prj, true); err != nil {
			return err
		}

		return i.jobRepo.RemoveByProject(ctx, projectID)
	}); err != nil {
		return err
	}

	// Only now is the project row durably gone: deleting the document earlier
	// would destroy it even if the transaction failed or was retried.
	if i.websocket != nil {
		if err := i.websocket.DeleteDocument(ctx, projectID.String()); err != nil {
			log.Errorfc(ctx, "project: could not delete websocket document for %s: %v", projectID, err)
		}
	}

	return nil
}

func (i *Project) Run(ctx context.Context, p interfaces.RunProjectParam) (_ *job.Job, err error) {
	ctx, span := otel.Tracer(tracerName).Start(ctx, "interactor.Project.Run")
	span.SetAttributes(attribute.String("project.id", p.ProjectID.String()))
	defer func() {
		if err != nil {
			span.RecordError(err)
		}
		span.End()
	}()

	proj, err := i.projectRepo.FindByID(ctx, p.ProjectID)
	if err != nil {
		return nil, err
	}
	if proj == nil {
		return nil, rerror.ErrNotFound
	}
	// A debug run executes the project without changing it, so anyone who can
	// see the project may run one. ActionRead is the only rule carrying
	// reader; ActionAny is writer and above, ActionEdit maintainer and above.
	if err := i.checkPermission(ctx, rbac.ActionRead, proj.Workspace()); err != nil {
		return nil, err
	}

	if p.Workflow == nil {
		return nil, nil
	}

	admission, err := i.planLimits.ensureCanRun(ctx, proj.Workspace())
	if err != nil {
		return nil, err
	}

	if err := i.websocket.FlushToGCS(ctx, p.ProjectID.String()); err != nil {
		return nil, err
	}

	useCloudRun := i.cloudRunWorker != nil

	doc, err := i.websocket.GetLatest(ctx, p.ProjectID.String())
	if err != nil {
		return nil, fmt.Errorf("failed to get latest project snapshot: %v", err)
	}
	projectID := p.ProjectID
	projectVersion := doc.Version

	debug := true

	j, err := job.New().
		NewID().
		Debug(&debug).
		ProjectID(&projectID).
		ProjectVersion(&projectVersion).
		Workspace(proj.Workspace()).
		Status(job.StatusPending).
		StartedAt(time.Now()).
		Build()
	if err != nil {
		return nil, err
	}

	workflowURL, err := i.file.UploadWorkflow(ctx, p.Workflow)
	if err != nil {
		return nil, err
	}
	workflowURLStr := workflowURL.String()

	metadataURL, err := i.file.UploadMetadata(ctx, j.ID().String(), []string{})
	if err != nil {
		return nil, fmt.Errorf("failed to upload metadata: %v", err)
	}
	if metadataURL != nil {
		j.SetMetadataURL(metadataURL.String())
	}

	j.SetParameters(p.Parameters)

	// Not in a transaction: the save reserves the plan's slot itself, and a
	// transaction would make it refuse runs that start together.
	if err := admission.save(ctx, j); err != nil {
		return nil, err
	}

	if !useCloudRun {
		// Submitted after the save, so a failure here leaves a job the user can
		// see rather than an untracked cloud job.
		gcpJobID, err := i.batch.SubmitJob(ctx, j.ID(), workflowURLStr, j.MetadataURL(), nil, p.ProjectID, proj.Workspace(), p.PreviousJobID, p.StartNodeID)
		if err != nil {
			failJob(ctx, i.jobRepo, j)
			return nil, fmt.Errorf("failed to submit job: %v", err)
		}
		j.SetGCPJobID(gcpJobID)

		if err := i.jobRepo.Save(ctx, j); err != nil {
			return nil, err
		}
	}

	if useCloudRun {
		if i.job != nil {
			// Run on Cloud Run; the standard monitoring loop finalizes the job.
			i.job.RunCloudRunWorker(j, gateway.RunJobParam{
				JobID:         j.ID(),
				WorkflowURL:   workflowURLStr,
				MetadataURL:   j.MetadataURL(),
				Variables:     nil,
				PreviousJobID: p.PreviousJobID,
				StartNodeID:   p.StartNodeID,
			})
			if err := i.job.StartMonitoring(ctx, j, nil); err != nil {
				return j, fmt.Errorf("failed to start job monitoring: %v", err)
			}
		}
	} else {
		if i.job != nil {
			if err := i.job.StartMonitoring(ctx, j, nil); err != nil {
				return j, fmt.Errorf("failed to start job monitoring: %v", err)
			}
		}
	}

	return j, nil
}

// previewSampleSizeCap bounds the per-source sample count to keep probe cost
// finite. A nil sampleSize is passed through so the engine applies its own
// default.
const previewSampleSizeCap = 1000

// PreviewSchema runs the engine's dynamic probe-schema over the live editor
// graph. It mirrors Run's orchestration (flush Yjs -> GCS, snapshot, persisted
// Job, Cloud Run worker dispatch with Batch fallback, monitoring) but is a fully
// dedicated code path: it never calls Run, tags the job Mode=preview-schema, does
// NOT upload metadata (the probe does not consume it), and dispatches through the
// worker's dedicated probe-schema route.
func (i *Project) PreviewSchema(ctx context.Context, p interfaces.PreviewSchemaParam) (_ *job.Job, err error) {
	ctx, span := otel.Tracer(tracerName).Start(ctx, "interactor.Project.PreviewSchema")
	span.SetAttributes(attribute.String("project.id", p.ProjectID.String()))
	defer func() {
		if err != nil {
			span.RecordError(err)
		}
		span.End()
	}()

	prj, err := i.projectRepo.FindByID(ctx, p.ProjectID)
	if err != nil {
		return nil, err
	}
	if prj == nil {
		return nil, rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionEdit, prj.Workspace()); err != nil {
		return nil, err
	}

	if p.Workflow == nil {
		return nil, interfaces.ErrWorkflowFileRequired
	}

	if err := i.websocket.FlushToGCS(ctx, p.ProjectID.String()); err != nil {
		return nil, err
	}

	variables := parametersToVariables(p.Parameters)
	sampleSize := capSampleSize(p.SampleSize)
	useCloudRun := i.cloudRunWorker != nil

	doc, err := i.websocket.GetLatest(ctx, p.ProjectID.String())
	if err != nil {
		return nil, fmt.Errorf("failed to get latest project snapshot: %v", err)
	}
	projectID := p.ProjectID
	projectVersion := doc.Version

	debug := true

	// Built before the transaction: the closure is retried on a serialization
	// failure, and a job ID minted inside it would submit a second probe job and
	// leave an orphaned workflow object on every attempt.
	j, err := job.New().
		NewID().
		Debug(&debug).
		Mode(job.ModePreviewSchema).
		ProjectID(&projectID).
		ProjectVersion(&projectVersion).
		Workspace(prj.Workspace()).
		Status(job.StatusPending).
		StartedAt(time.Now()).
		Build()
	if err != nil {
		return nil, err
	}

	workflowURL, err := i.file.UploadWorkflow(ctx, p.Workflow)
	if err != nil {
		return nil, err
	}
	workflowURLStr := workflowURL.String()

	// Intentionally NO UploadMetadata: probe-schema does not consume metadata.

	j.SetParameters(p.Parameters)
	// The report URL is surfaced via outputURLs on completion by
	// Job.updateJobArtifacts, not here: the artifact does not exist until the
	// worker writes it (and never on failure), so setting it at creation time
	// would hand clients a URL that 404s.

	if err := i.transaction.WithinTransaction(ctx, func(ctx context.Context) error {
		return i.jobRepo.Save(ctx, j)
	}); err != nil {
		return nil, err
	}

	reportURL := i.file.GetJobPreviewSchemaUploadURI(j.ID().String())

	if !useCloudRun {
		// Submitted after commit so a retry cannot double-dispatch, and a failure
		// here leaves a job the user can see rather than an untracked probe job.
		gcpJobID, err := i.batch.SubmitProbeJob(ctx, j.ID(), workflowURLStr, variables, sampleSize, reportURL, p.ProjectID, prj.Workspace())
		if err != nil {
			failJob(ctx, i.jobRepo, j)
			return nil, fmt.Errorf("failed to submit probe job: %v", err)
		}
		j.SetGCPJobID(gcpJobID)

		if err := i.jobRepo.Save(ctx, j); err != nil {
			return nil, err
		}
	}

	if useCloudRun {
		if i.job != nil {
			// Dispatch on Cloud Run via the dedicated probe-schema route; the
			// standard monitoring loop finalizes the job.
			i.job.PreviewSchemaCloudRunWorker(j, gateway.ProbeSchemaParam{
				JobID:       j.ID(),
				WorkflowURL: workflowURLStr,
				ReportURL:   reportURL,
				Variables:   variables,
				SampleSize:  sampleSize,
			})
			if err := i.job.StartMonitoring(ctx, j, nil); err != nil {
				return j, fmt.Errorf("failed to start job monitoring: %v", err)
			}
		}
	} else {
		if i.job != nil {
			if err := i.job.StartMonitoring(ctx, j, nil); err != nil {
				return j, fmt.Errorf("failed to start job monitoring: %v", err)
			}
		}
	}

	return j, nil
}

// capSampleSize bounds an optional sample size to keep probe cost finite. A nil
// input is passed through so the engine applies its own default.
func capSampleSize(s *int) *int {
	if s == nil {
		return nil
	}
	v := *s
	if v < 1 {
		v = 1
	}
	if v > previewSampleSizeCap {
		v = previewSampleSizeCap
	}
	return &v
}

// parametersToVariables renders project parameters as engine --var key/values.
func parametersToVariables(params []*parameter.Parameter) map[string]string {
	if len(params) == 0 {
		return nil
	}
	vars := make(map[string]string, len(params))
	for _, p := range params {
		if p == nil {
			continue
		}
		// A parameter with no default must not be sent: stringifying nil yields
		// the literal "<nil>", which would override the workflow's variables.get(...)
		// resolution / the engine default. Leave it unset instead.
		dv := p.DefaultValue()
		if dv == nil {
			continue
		}
		vars[p.Name()] = fmt.Sprintf("%v", dv)
	}
	return vars
}
