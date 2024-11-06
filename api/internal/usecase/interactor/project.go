package interactor

import (
	"context"
	"fmt"

	"github.com/reearth/reearth-flow/api/internal/usecase"
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
	common
	assetRepo     repo.Asset
	workflowRepo  repo.Workflow
	projectRepo   repo.Project
	userRepo      accountrepo.User
	workspaceRepo accountrepo.Workspace
	transaction   usecasex.Transaction
	file          gateway.File
	batch         gateway.Batch
}

func NewProject(r *repo.Container, gr *gateway.Container) interfaces.Project {
	return &Project{
		assetRepo:     r.Asset,
		workflowRepo:  r.Workflow,
		projectRepo:   r.Project,
		userRepo:      r.User,
		workspaceRepo: r.Workspace,
		transaction:   r.Transaction,
		file:          gr.File,
	}
}

func (i *Project) Fetch(ctx context.Context, ids []id.ProjectID, _ *usecase.Operator) ([]*project.Project, error) {
	return i.projectRepo.FindByIDs(ctx, ids)
}

func (i *Project) FindByWorkspace(ctx context.Context, id accountdomain.WorkspaceID, p *usecasex.Pagination, _ *usecase.Operator) ([]*project.Project, *usecasex.PageInfo, error) {
	return i.projectRepo.FindByWorkspace(ctx, id, p)
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

func (i *Project) Run(ctx context.Context, p interfaces.RunProjectParam, operator *usecase.Operator) (started bool, err error) {
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

	// prj, err := i.projectRepo.FindByID(ctx, p.Workflow.Project)
	// if err != nil {
	// 	return false, err
	// }

	// if err := i.CanWriteWorkspace(prj.Workspace(), operator); err != nil {
	// 	return false, err
	// }

	// prevWf, _ := i.workflowRepo.FindByID(ctx, prj.Workspace(), prj.Workflow())
	// if prevWf != nil && prevWf.ID == prj.Workflow() {
	// 	if err := i.workflowRepo.Remove(ctx, prevWf.Workspace, prevWf.ID); err != nil {
	// 		return false, err
	// 	}
	// }

	// if err := i.workflowRepo.Save(ctx, prj.Workspace(), p.Workflow); err != nil {
	// 	return false, err
	// }

	// prj.UpdateWorkflow(p.Workflow.ID)
	// if err := i.projectRepo.Save(ctx, prj); err != nil {
	// 	return false, err
	// }

	jobID := id.NewJobID()
	_, err = i.batch.SubmitJob(ctx, jobID, p.Workflow.Path, p.ProjectID)
	if err != nil {
		return false, fmt.Errorf("failed to submit job: %v", err)
	}

	tx.Commit()
	return true, nil
}
