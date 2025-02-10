package interactor

import (
	"context"
	"errors"
	"fmt"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/projectAccess"
	"github.com/reearth/reearthx/usecasex"
)

type ProjectAccess struct {
	common
	projectRepo       repo.Project
	projectAccessRepo repo.ProjectAccess
	transaction       usecasex.Transaction
}

func NewProjectAccess(r *repo.Container, gr *gateway.Container) interfaces.Project {
	return &Project{
		projectRepo: r.Project,
		transaction: r.Transaction,
	}
}
func (i *ProjectAccess) Share(ctx context.Context, projectID id.ProjectID, operator *usecase.Operator) (sharingUrl string, err error) {
	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return "", err
	}

	ctx = tx.Context()
	defer func() {
		if err2 := tx.End(ctx); err == nil && err2 != nil {
			err = err2
		}
	}()

	prj, err := i.projectRepo.FindByID(ctx, projectID)
	if err != nil {
		return "", err
	}
	if err := i.CanWriteWorkspace(prj.Workspace(), operator); err != nil {
		return "", err
	}

	var pa *projectAccess.ProjectAccess
	pa, err = i.projectAccessRepo.FindByProjectID(ctx, projectID)
	if err != nil {
		return "", fmt.Errorf("failed to find project access: %w", err)
	}

	if pa == nil {
		pa, err = projectAccess.New().
			NewID().
			Project(prj.ID()).
			Build()
		if err != nil {
			return "", err
		}
	}

	err = pa.MakePublic()
	if err != nil {
		return "", err
	}

	err = i.projectAccessRepo.Save(ctx, pa)
	if err != nil {
		return "", err
	}

	sharingUrl, err = pa.SharingURL("https://your-domain.com")
	if err != nil {
		return "", err
	}
	return sharingUrl, nil
}

func (i *ProjectAccess) Unshare(ctx context.Context, projectID id.ProjectID, operator *usecase.Operator) (err error) {
	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return err
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

	pa, err := i.projectAccessRepo.FindByProjectID(ctx, projectID)
	if err != nil {
		return fmt.Errorf("failed to find project access: %w", err)
	}
	if pa == nil {
		return errors.New("project access not found")
	}

	pa.MakePrivate()

	err = i.projectAccessRepo.Save(ctx, pa)
	if err != nil {
		return err
	}

	return nil
}
