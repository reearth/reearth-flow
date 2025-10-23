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
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearth-flow/api/pkg/workspace"
	"github.com/reearth/reearthx/usecasex"
)

type Project struct {
	assetRepo         repo.Asset
	workflowRepo      repo.Workflow
	projectRepo       repo.Project
	jobRepo           repo.Job
	workspaceRepo     workspace.Repo
	transaction       usecasex.Transaction
	file              gateway.File
	batch             gateway.Batch
	job               interfaces.Job
	permissionChecker gateway.PermissionChecker
}

func NewProject(r *repo.Container, gr *gateway.Container, jobUsecase interfaces.Job, permissionChecker gateway.PermissionChecker, workspaceRepo workspace.Repo) interfaces.Project {
	return &Project{
		assetRepo:         r.Asset,
		workflowRepo:      r.Workflow,
		projectRepo:       r.Project,
		jobRepo:           r.Job,
		workspaceRepo:     workspaceRepo,
		transaction:       r.Transaction,
		file:              gr.File,
		batch:             gr.Batch,
		job:               jobUsecase,
		permissionChecker: permissionChecker,
	}
}

func (i *Project) checkPermission(ctx context.Context, action string) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceProject, action)
}

func (i *Project) Fetch(ctx context.Context, ids []id.ProjectID) ([]*project.Project, error) {
	if err := i.checkPermission(ctx, rbac.ActionList); err != nil {
		return nil, err
	}

	return i.projectRepo.FindByIDs(ctx, ids)
}

func (i *Project) FindByWorkspace(ctx context.Context, id id.WorkspaceID, pagination *interfaces.PaginationParam, keyword *string) ([]*project.Project, *interfaces.PageBasedInfo, error) {
	if err := i.checkPermission(ctx, rbac.ActionList); err != nil {
		return nil, nil, err
	}

	return i.projectRepo.FindByWorkspace(ctx, id, pagination, keyword)
}

func (i *Project) Create(ctx context.Context, p interfaces.CreateProjectParam) (_ *project.Project, err error) {
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

	_, err = i.workspaceRepo.FindByID(ctx, p.WorkspaceID)
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

	tx.Commit()
	return nil
}

func (i *Project) Run(ctx context.Context, p interfaces.RunProjectParam) (_ *job.Job, err error) {
	if err := i.checkPermission(ctx, rbac.ActionEdit); err != nil {
		return nil, err
	}

	if p.Workflow == nil {
		return nil, nil
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

	debug := true

	j, err := job.New().
		NewID().
		Debug(&debug).
		Deployment(id.NewDeploymentID()). // Using a placeholder deployment ID
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

	if err := i.jobRepo.Save(ctx, j); err != nil {
		return nil, err
	}

	gcpJobID, err := i.batch.SubmitJob(ctx, j.ID(), workflowURL.String(), j.MetadataURL(), nil, p.ProjectID, prj.Workspace())
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

	return j, nil
}
