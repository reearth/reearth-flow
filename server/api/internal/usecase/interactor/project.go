package interactor

import (
	"context"
	"fmt"
	"time"

	gqlworkspace "github.com/reearth/reearth-accounts/server/pkg/gqlclient/workspace"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
)

type Project struct {
	assetRepo         repo.Asset
	workflowRepo      repo.Workflow
	projectRepo       repo.Project
	jobRepo           repo.Job
	workerConfigRepo  repo.WorkerConfig
	workspaceRepo     gqlworkspace.WorkspaceRepo
	transaction       usecasex.Transaction
	file              gateway.File
	batch             gateway.Batch
	cloudRunWorker    gateway.CloudRunWorker
	websocket         interfaces.WebsocketClient
	job               interfaces.Job
	permissionChecker gateway.PermissionChecker
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
	}
}

func (i *Project) checkPermission(ctx context.Context, action string, workspaceID ...accountsid.WorkspaceID) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceProject, action, workspaceID...)
}

func (i *Project) Fetch(ctx context.Context, ids []id.ProjectID) ([]*project.Project, error) {
	projects, err := i.projectRepo.FindByIDs(ctx, ids)
	if err != nil {
		return nil, err
	}

	if len(projects) == 0 {
		if err := i.checkPermission(ctx, rbac.ActionList); err != nil {
			return nil, err
		}
	} else {
		if err := i.checkPermission(ctx, rbac.ActionList, projects[0].Workspace()); err != nil { // single-workspace batch assumption
			return nil, err
		}
	}

	return projects, nil
}

func (i *Project) FindByWorkspace(ctx context.Context, id accountsid.WorkspaceID, pagination *interfaces.PaginationParam, keyword *string, includeArchived *bool) ([]*project.Project, *interfaces.PageBasedInfo, error) {
	if err := i.checkPermission(ctx, rbac.ActionList, id); err != nil {
		return nil, nil, err
	}

	return i.projectRepo.FindByWorkspace(ctx, id, pagination, keyword, includeArchived)
}

func (i *Project) Create(ctx context.Context, p interfaces.CreateProjectParam) (_ *project.Project, err error) {
	if err := i.checkPermission(ctx, rbac.ActionCreate, p.WorkspaceID); err != nil {
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

	_, err = i.workspaceRepo.FindByID(ctx, p.WorkspaceID.String())
	if err != nil {
		return nil, err
	}

	pb := project.New().
		NewID().
		Workspace(p.WorkspaceID)
	if p.Name != nil {
		pb = pb.Name(*p.Name)
	}
	if p.Description != nil {
		pb = pb.Description(*p.Description)
	}
	if p.Archived != nil {
		pb = pb.IsArchived(*p.Archived)
	}

	proj, err := pb.Build()
	if err != nil {
		return nil, err
	}

	err = i.projectRepo.Save(ctx, proj)
	if err != nil {
		return nil, err
	}

	tx.Commit()
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

	prj, err := i.projectRepo.FindByID(ctx, p.ID)
	if err != nil {
		return nil, err
	}

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

	if err := i.projectRepo.Save(ctx, prj); err != nil {
		return nil, err
	}

	tx.Commit()
	return prj, nil
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

	prj, err := i.projectRepo.FindByID(ctx, projectID)
	if err != nil {
		return err
	}

	deleter := ProjectDeleter{
		File:      i.file,
		Project:   i.projectRepo,
		Websocket: i.websocket,
	}
	if err := deleter.Delete(ctx, prj, true); err != nil {
		return err
	}

	if err := i.jobRepo.RemoveByProject(ctx, projectID); err != nil {
		return err
	}

	tx.Commit()
	return nil
}

func (i *Project) Run(ctx context.Context, p interfaces.RunProjectParam) (_ *job.Job, err error) {
	proj, err := i.projectRepo.FindByID(ctx, p.ProjectID)
	if err != nil {
		return nil, err
	}
	if proj == nil {
		return nil, rerror.ErrNotFound
	}
	if err := i.checkPermission(ctx, rbac.ActionEdit, proj.Workspace()); err != nil {
		return nil, err
	}

	if p.Workflow == nil {
		return nil, nil
	}

	if err := i.websocket.FlushToGCS(ctx, p.ProjectID.String()); err != nil {
		return nil, err
	}

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return nil, err
	}

	ctx = tx.Context()
	defer func() {
		if err2 := tx.End(ctx); err == nil && err2 != nil {
			err = err2
		}
	}()

	prj, err := i.projectRepo.FindByID(ctx, p.ProjectID)
	if err != nil {
		return nil, err
	}

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

	metadataURL, err := i.file.UploadMetadata(ctx, j.ID().String(), []string{})
	if err != nil {
		return nil, fmt.Errorf("failed to upload metadata: %v", err)
	}
	if metadataURL != nil {
		j.SetMetadataURL(metadataURL.String())
	}

	j.SetParameters(p.Parameters)

	if err := i.jobRepo.Save(ctx, j); err != nil {
		return nil, err
	}

	if i.cloudRunWorker != nil {
		tx.Commit()
		if i.job != nil {
			// Run on Cloud Run; the standard monitoring loop finalizes the job.
			i.job.RunCloudRunWorker(j, gateway.RunJobParam{
				JobID:         j.ID(),
				WorkflowURL:   workflowURL.String(),
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
		gcpJobID, err := i.batch.SubmitJob(ctx, j.ID(), workflowURL.String(), j.MetadataURL(), nil, p.ProjectID, prj.Workspace(), p.PreviousJobID, p.StartNodeID)
		if err != nil {
			return nil, fmt.Errorf("failed to submit job: %v", err)
		}
		j.SetGCPJobID(gcpJobID)

		if err := i.jobRepo.Save(ctx, j); err != nil {
			return nil, err
		}

		tx.Commit()

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
	if err := i.checkPermission(ctx, rbac.ActionEdit); err != nil {
		return nil, err
	}

	if p.Workflow == nil {
		return nil, interfaces.ErrWorkflowFileRequired
	}

	if err := i.websocket.FlushToGCS(ctx, p.ProjectID.String()); err != nil {
		return nil, err
	}

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return nil, err
	}

	ctx = tx.Context()
	defer func() {
		if err2 := tx.End(ctx); err == nil && err2 != nil {
			err = err2
		}
	}()

	prj, err := i.projectRepo.FindByID(ctx, p.ProjectID)
	if err != nil {
		return nil, err
	}

	doc, err := i.websocket.GetLatest(ctx, p.ProjectID.String())
	if err != nil {
		return nil, fmt.Errorf("failed to get latest project snapshot: %v", err)
	}
	projectID := p.ProjectID
	projectVersion := doc.Version

	debug := true
	sampleSize := capSampleSize(p.SampleSize)

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

	// Intentionally NO UploadMetadata: probe-schema does not consume metadata.

	j.SetParameters(p.Parameters)
	// The report URL is surfaced via outputURLs on completion by
	// Job.updateJobArtifacts, not here: the artifact does not exist until the
	// worker writes it (and never on failure), so setting it at creation time
	// would hand clients a URL that 404s.

	if err := i.jobRepo.Save(ctx, j); err != nil {
		return nil, err
	}

	variables := parametersToVariables(p.Parameters)
	reportURL := i.file.GetJobPreviewSchemaUploadURI(j.ID().String())

	if i.cloudRunWorker != nil {
		tx.Commit()
		if i.job != nil {
			// Dispatch on Cloud Run via the dedicated probe-schema route; the
			// standard monitoring loop finalizes the job.
			i.job.PreviewSchemaCloudRunWorker(j, gateway.ProbeSchemaParam{
				JobID:       j.ID(),
				WorkflowURL: workflowURL.String(),
				ReportURL:   reportURL,
				Variables:   variables,
				SampleSize:  sampleSize,
			})
			if err := i.job.StartMonitoring(ctx, j, nil); err != nil {
				return j, fmt.Errorf("failed to start job monitoring: %v", err)
			}
		}
	} else {
		gcpJobID, err := i.batch.SubmitProbeJob(ctx, j.ID(), workflowURL.String(), variables, sampleSize, reportURL, p.ProjectID, prj.Workspace())
		if err != nil {
			return nil, fmt.Errorf("failed to submit probe job: %v", err)
		}
		j.SetGCPJobID(gcpJobID)

		if err := i.jobRepo.Save(ctx, j); err != nil {
			return nil, err
		}

		tx.Commit()

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
		// the literal "<nil>", which would override the workflow's env.get(...)
		// resolution / the engine default. Leave it unset instead.
		dv := p.DefaultValue()
		if dv == nil {
			continue
		}
		vars[p.Name()] = fmt.Sprintf("%v", dv)
	}
	return vars
}
