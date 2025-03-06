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
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/usecasex"
)

type Project struct {
	common
	assetRepo     repo.Asset
	workflowRepo  repo.Workflow
	projectRepo   repo.Project
	jobRepo       repo.Job
	userRepo      accountrepo.User
	workspaceRepo accountrepo.Workspace
	transaction   usecasex.Transaction
	file          gateway.File
	batch         gateway.Batch
	job           interfaces.Job
}

func NewProject(r *repo.Container, gr *gateway.Container, jobUsecase interfaces.Job) interfaces.Project {
	return &Project{
		assetRepo:     r.Asset,
		workflowRepo:  r.Workflow,
		projectRepo:   r.Project,
		jobRepo:       r.Job,
		userRepo:      r.User,
		workspaceRepo: r.Workspace,
		transaction:   r.Transaction,
		file:          gr.File,
		batch:         gr.Batch,
		job:           jobUsecase,
	}
}

func (i *Project) Fetch(ctx context.Context, ids []id.ProjectID, _ *usecase.Operator) ([]*project.Project, error) {
	return i.projectRepo.FindByIDs(ctx, ids)
}

func (i *Project) FindByWorkspace(ctx context.Context, id accountdomain.WorkspaceID, pagination *interfaces.PaginationParam, _ *usecase.Operator) ([]*project.Project, *interfaces.PageBasedInfo, error) {
	return i.projectRepo.FindByWorkspace(ctx, id, pagination)
}

func (i *Project) Create(ctx context.Context, p interfaces.CreateProjectParam, operator *usecase.Operator) (_ *project.Project, err error) {
	if err := i.CanWriteWorkspace(p.WorkspaceID, operator); err != nil {
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

func (i *Project) Update(ctx context.Context, p interfaces.UpdateProjectParam, operator *usecase.Operator) (_ *project.Project, err error) {
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
	if err := i.CanWriteWorkspace(prj.Workspace(), operator); err != nil {
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

func (i *Project) Delete(ctx context.Context, projectID id.ProjectID, operator *usecase.Operator) (err error) {
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
	if err := i.CanWriteWorkspace(prj.Workspace(), operator); err != nil {
		return err
	}

	deleter := ProjectDeleter{
		File:    i.file,
		Project: i.projectRepo,
	}
	if err := deleter.Delete(ctx, prj, true, operator); err != nil {
		return err
	}

	tx.Commit()
	return nil
}

func (i *Project) Run(ctx context.Context, p interfaces.RunProjectParam, operator *usecase.Operator) (_ *job.Job, err error) {
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

	gcpJobID, err := i.batch.SubmitJob(ctx, j.ID(), p.Workflow.Path, j.MetadataURL(), nil, p.ProjectID, prj.Workspace())
	if err != nil {
		return nil, fmt.Errorf("failed to submit job: %v", err)
	}
	j.SetGCPJobID(gcpJobID)

	if err := i.jobRepo.Save(ctx, j); err != nil {
		return nil, err
	}

	tx.Commit()

	if i.job != nil {
		if err := i.job.StartMonitoring(ctx, j, nil, operator); err != nil {
			return nil, fmt.Errorf("failed to start job monitoring: %v", err)
		}
	}

	return j, nil
}
