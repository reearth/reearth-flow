package interactor

import (
	"context"
	"fmt"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/usecasex"
)

type Project struct {
	assetRepo         repo.Asset
	workflowRepo      repo.Workflow
	projectRepo       repo.Project
	userRepo          accountrepo.User
	workspaceRepo     accountrepo.Workspace
	transaction       usecasex.Transaction
	file              gateway.File
	batch             gateway.Batch
	permissionChecker gateway.PermissionChecker
}

func NewProject(r *repo.Container, gr *gateway.Container, permissionChecker gateway.PermissionChecker) interfaces.Project {
	return &Project{
		assetRepo:         r.Asset,
		workflowRepo:      r.Workflow,
		projectRepo:       r.Project,
		userRepo:          r.User,
		workspaceRepo:     r.Workspace,
		transaction:       r.Transaction,
		file:              gr.File,
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

func (i *Project) FindByWorkspace(ctx context.Context, id accountdomain.WorkspaceID, pagination *interfaces.PaginationParam) ([]*project.Project, *interfaces.PageBasedInfo, error) {
	if err := i.checkPermission(ctx, rbac.ActionList); err != nil {
		return nil, nil, err
	}

	return i.projectRepo.FindByWorkspace(ctx, id, pagination)
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

func (i *Project) Run(ctx context.Context, p interfaces.RunProjectParam) (started bool, err error) {
	if err := i.checkPermission(ctx, rbac.ActionEdit); err != nil {
		return false, err
	}

	if p.Workflow == nil {
		return false, nil
	}

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return false, err
	}

	ctx = tx.Context()
	defer func() {
		if err2 := tx.End(ctx); err == nil && err2 != nil {
			err = err2
		}
	}()

	fmt.Println("RunProjectParam", p)

	prj, err := i.projectRepo.FindByID(ctx, p.ProjectID)
	if err != nil {
		return false, err
	}

	jobID := id.NewJobID()
	_, err = i.batch.SubmitJob(ctx, jobID, p.Workflow.Path, "", nil, p.ProjectID, prj.Workspace())
	if err != nil {
		return false, fmt.Errorf("failed to submit job: %v", err)
	}

	tx.Commit()
	return true, nil
}
